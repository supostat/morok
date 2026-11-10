use std::env;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::process;

use chromiumoxide::Page;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetDeviceMetricsOverrideParams};
use chromiumoxide::cdp::browser_protocol::page::Viewport;
use chromiumoxide::fetcher::{BrowserFetcher, BrowserFetcherOptions, BrowserKind, Version};
use chromiumoxide::page::ScreenshotParams;
use futures::StreamExt;
use morok_core::{Rect, Size};
use morok_style::ColorScheme;
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;

use crate::case::Case;
use crate::case_rendering::CaseRendering;
use crate::chrome_build::ChromeBuild;
use crate::compiled_suite::CompiledSuite;
use crate::frame_png;
use crate::markup::MOTION_VARIABLE;
use crate::rendering::{DEVICE_SCALE, Rendering};
use crate::scene::Scene;
use crate::stand_error::StandError;

/// Where the builds are kept between runs, under the home directory of the user.
const CACHE_DIRECTORY: &str = "Library/Caches/morok-teststand/chrome";
const VERSION_PARTS: usize = 4;
const BLANK_PAGE: &str = "about:blank";
const COLOR_SCHEME_FEATURE: &str = "prefers-color-scheme";
/// Colors are drawn as the sRGB values the page gives, whatever the profile of the display,
/// and by the software rasterizer, so that a build draws a page the same on every run.
const ARGUMENTS: [&str; 2] = ["force-color-profile=srgb", "disable-gpu"];
/// The scale is that of the whole browser, as on a display of this density. A scale emulated
/// for one page leaves layout in CSS pixels, where Chrome rounds lines and borders to whole
/// CSS pixels; on a dense display it rounds them to device pixels.
const SCALE_ARGUMENT: &str = "force-device-scale-factor";
/// Leaves the scale of the browser as it is when the size of a page is set.
const NO_SCALE_OVERRIDE: f64 = 0.0;
/// Fonts and pictures are loaded before the page is read; a picture that failed is loaded too.
const FONTS_ARE_LOADED: &str = "Promise.all([document.fonts.ready, \
    ...Array.from(document.images, image => image.complete ? Promise.resolve() : \
    new Promise(resolve => { image.onload = resolve; image.onerror = resolve; }))]).then(() => true)";
/// Plays the motion of a case when its page has one: once the page is styled as it loaded, the
/// elements take the classes of the change, which starts their transitions, and every animation
/// is paused at the time the case is drawn at. A frame is let through before the page is read.
const PLAY_MOTION: &str = "(async () => {\
    const motion = window.MOTION_VARIABLE;\
    if (!motion) { return true; }\
    document.body.getBoundingClientRect();\
    const elements = Array.from(document.body.querySelectorAll('div, svg, img'));\
    elements.forEach((element, index) => element.setAttribute('class', motion.classes[index]));\
    for (const animation of document.getAnimations()) {\
    animation.pause();\
    animation.currentTime = motion.milliseconds;\
    }\
    await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));\
    return true;\
    })()";
/// The elements of the tree: the `div` of an element, the `svg` of an icon and the `img` of an
/// image, in document order; what an SVG holds inside is no element of the tree.
const ELEMENT_BOXES: &str = "Array.from(document.body.querySelectorAll('div, svg, img'), element => {\
    const box = element.getBoundingClientRect();\
    return [box.x, box.y, box.width, box.height];\
    })";

/// The pinned Chrome as the stand drives it: headless, over its debugging protocol.
#[derive(Debug)]
pub struct ChromeSide {
    runtime: Runtime,
    browser: Browser,
    /// Passes the messages of the browser on for as long as it runs.
    events: JoinHandle<()>,
    profile_directory: PathBuf,
}

impl ChromeSide {
    /// Launches the build, fetching it into the cache of the user first when it is not there.
    /// The Chrome installed on the system is never used.
    pub fn launch(build: &ChromeBuild) -> Result<Self, StandError> {
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(chrome_error)?;
        let profile_directory = env::temp_dir().join(format!("morok-teststand-chrome-{}", process::id()));
        let (browser, events) = runtime.block_on(async {
            let executable = fetched_executable(build).await?;
            let mut configuration = BrowserConfig::builder()
                .chrome_executable(executable)
                .new_headless_mode()
                .user_data_dir(&profile_directory)
                .viewport(None);
            for argument in ARGUMENTS {
                configuration = configuration.arg(argument);
            }
            configuration = configuration.arg(format!("{SCALE_ARGUMENT}={DEVICE_SCALE}"));
            let (browser, mut handler) = Browser::launch(configuration.build().map_err(chrome_error)?)
                .await
                .map_err(chrome_error)?;
            let events = tokio::spawn(async move { while handler.next().await.is_some_and(|event| event.is_ok()) {} });
            Ok::<_, StandError>((browser, events))
        })?;
        Ok(Self {
            runtime,
            browser,
            events,
            profile_directory,
        })
    }

    /// Draws the case under both color schemes, at its moment. Each color scheme gets a page of
    /// its own, loaded under that scheme: switching a page to another scheme would start the
    /// transitions of the colors that change with it.
    pub fn render_case(&mut self, suite: &CompiledSuite<'_>, case: &Case) -> Result<CaseRendering, StandError> {
        let scene = suite.scene(case, ColorScheme::Light);
        let document = suite.page(case);
        Ok(CaseRendering {
            light: self.render(&scene, &document, case.viewport, ColorScheme::Light)?,
            dark: self.render(&scene, &document, case.viewport, ColorScheme::Dark)?,
        })
    }

    fn render(
        &mut self,
        scene: &Scene,
        document: &str,
        viewport: Size,
        color_scheme: ColorScheme,
    ) -> Result<Rendering, StandError> {
        self.runtime.block_on(async {
            let page = self.browser.new_page(BLANK_PAGE).await.map_err(chrome_error)?;
            let (width, height) = whole_pixels(viewport);
            page.execute(SetDeviceMetricsOverrideParams::new(
                width,
                height,
                NO_SCALE_OVERRIDE,
                false,
            ))
            .await
            .map_err(chrome_error)?;
            page.emulate_media_features(vec![MediaFeature::new(COLOR_SCHEME_FEATURE, color_scheme.to_string())])
                .await
                .map_err(chrome_error)?;
            page.set_content(document).await.map_err(chrome_error)?;
            page.evaluate(FONTS_ARE_LOADED).await.map_err(chrome_error)?;
            page.evaluate(PLAY_MOTION.replace("MOTION_VARIABLE", MOTION_VARIABLE))
                .await
                .map_err(chrome_error)?;
            let rendering = read_page(&page, scene, viewport).await?;
            page.close().await.map_err(chrome_error)?;
            Ok(rendering)
        })
    }

    /// Quits the browser, waiting until it is gone.
    pub fn close(mut self) -> Result<(), StandError> {
        self.runtime.block_on(async {
            self.browser.close().await.map_err(chrome_error)?;
            self.browser.wait().await.map_err(chrome_error)?;
            (&mut self.events).await.map_err(chrome_error)
        })
    }
}

impl Drop for ChromeSide {
    /// Stops a browser that was not closed, as after an error half way through a suite, and
    /// drops the profile it ran with. Neither can fail in a way the stand could act on: a
    /// browser that has quit is no longer there to be stopped, and the profile lies in the
    /// temporary directory of the system.
    fn drop(&mut self) {
        self.runtime.block_on(async {
            let _ = self.browser.kill().await;
            let _ = self.browser.wait().await;
        });
        let _ = fs::remove_dir_all(&self.profile_directory);
    }
}

/// The executable of the build in the cache of the user; the build is downloaded on the first use.
async fn fetched_executable(build: &ChromeBuild) -> Result<PathBuf, StandError> {
    let version = exact_version(build)?;
    let home = env::home_dir().ok_or_else(|| chrome_error("the user has no home directory to cache the build in"))?;
    let cache = home.join(CACHE_DIRECTORY);
    fs::create_dir_all(&cache).map_err(|error| StandError::File {
        path: cache.clone(),
        error,
    })?;
    let options = BrowserFetcherOptions::builder()
        .with_path(&cache)
        .with_kind(BrowserKind::Chrome)
        .with_version(version)
        .build()
        .map_err(chrome_error)?;
    let installation = BrowserFetcher::new(options).fetch().await.map_err(chrome_error)?;
    Ok(installation.executable_path)
}

/// A build is pinned by all four parts of its version: a shorter one names whatever build is
/// the latest of a branch.
fn exact_version(build: &ChromeBuild) -> Result<Version, StandError> {
    let version: Version = build.version.parse().map_err(chrome_error)?;
    if build.version.split('.').count() == VERSION_PARTS {
        Ok(version)
    } else {
        Err(chrome_error(format!(
            "`{}` does not pin a build: a version has four parts",
            build.version
        )))
    }
}

/// The picture of the page and the boxes of its elements.
async fn read_page(page: &Page, scene: &Scene, viewport: Size) -> Result<Rendering, StandError> {
    let screenshot = page
        .screenshot(
            ScreenshotParams::builder()
                .omit_background(true)
                .clip(Viewport {
                    x: 0.0,
                    y: 0.0,
                    width: f64::from(viewport.width),
                    height: f64::from(viewport.height),
                    scale: 1.0,
                })
                .build(),
        )
        .await
        .map_err(chrome_error)?;
    let rectangles: Vec<[f32; 4]> = page
        .evaluate(ELEMENT_BOXES)
        .await
        .map_err(chrome_error)?
        .into_value()
        .map_err(chrome_error)?;
    let element_count = scene.elements().len();
    if rectangles.len() != element_count {
        return Err(chrome_error(format!(
            "the page holds {} elements and the tree {element_count}",
            rectangles.len()
        )));
    }
    Ok(Rendering {
        image: frame_png::decode(&screenshot).map_err(chrome_error)?,
        boxes: scene.element_boxes(
            rectangles
                .into_iter()
                .map(|[x, y, width, height]| Rect::new(x, y, width, height)),
        ),
    })
}

/// The size of the surface as the whole pixels the protocol takes.
///
/// # Panics
///
/// When a side of the surface is not a whole number of pixels.
fn whole_pixels(viewport: Size) -> (i64, i64) {
    assert!(
        viewport.width.fract() == 0.0 && viewport.height.fract() == 0.0,
        "a case is drawn on a surface of whole pixels, not {viewport:?}"
    );
    (viewport.width as i64, viewport.height as i64)
}

fn chrome_error(reason: impl Display) -> StandError {
    StandError::Chrome(reason.to_string())
}
