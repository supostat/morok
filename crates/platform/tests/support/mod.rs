#![allow(dead_code)]

use std::process::ExitCode;
use std::time::Duration;

use morok_core::accesskit::{ActionRequest, TreeUpdate};
use morok_core::{ElementId, ElementTree, FrameScheduler, KeyEvent, PointerInput, Rect, Role, ScrollInput, Size};
use morok_css::{CompiledTokens, SourceFile, compile_stylesheets, compile_tokens};
use morok_platform::{CoreTextShaper, FrameImage, OffscreenTarget, Renderer, SurfaceContent, SurfaceSettings};
use morok_style::{ColorScheme, StyleSheet, Theme};
use objc2_foundation::{NSDate, NSRunLoop};

/// The tokens of the scenes: pure colors, so that a pixel tells which item drew it.
const TOKENS: &str = "\
    --color-paper: #FFFFFF; --color-ink: #000000; --color-red: #FF0000; --color-green: #00FF00; \
    --color-blue: #0000FF; --color-veil: rgba(0, 0, 255, 0.5); \
    --radius-corner: 20px; --radius-small: 10px; --radius-large: 40px; \
    --shadow-soft: 0 0 20px #000000; --shadow-hard: 10px 6px 0 4px rgba(0, 0, 0, 0.5); \
    --shadow-well: inset 0 0 20px #000000; --shadow-rim: inset 0 0 0 5px #FF0000; \
    --font-ui: system-ui; --font-mono: ui-monospace; \
    --text-md: 16px; --text-lg: 32px; --text-leading: 1.5; --text-weight-bold: 700;";

/// An element tree styled by a stylesheet the real compiler compiled, on a surface of its own.
pub(crate) struct Scene {
    pub(crate) tree: ElementTree,
    pub(crate) viewport: Size,
    /// Device pixels to a CSS pixel on the surface the scene is laid out for.
    pub(crate) scale: f32,
    /// The time of the next frame.
    pub(crate) time: Duration,
    pub(crate) text_shaper: CoreTextShaper,
    tokens: CompiledTokens,
    style_sheet: StyleSheet,
}

impl Scene {
    /// A scene on a surface of `viewport`; `rules` is the text of its stylesheet.
    pub(crate) fn new(rules: &str, viewport: Size) -> Self {
        let tokens_source =
            format!(":root {{ {TOKENS} }}\n@media (prefers-color-scheme: dark) {{ :root {{ {TOKENS} }} }}\n");
        let tokens = compile_tokens(&SourceFile::new("tokens.css", tokens_source))
            .unwrap_or_else(|diagnostics| panic!("the tokens compile:\n{diagnostics}"));
        let style_sheet = compile_stylesheets(&[SourceFile::new("scene.css", rules)], &tokens)
            .unwrap_or_else(|diagnostics| panic!("the stylesheet compiles:\n{diagnostics}"));
        Self {
            tree: ElementTree::new(style_sheet.clone(), tokens.light.clone()),
            viewport,
            scale: 1.0,
            time: Duration::ZERO,
            text_shaper: CoreTextShaper::new(),
            tokens,
            style_sheet,
        }
    }

    pub(crate) fn root(&self) -> ElementId {
        self.tree.root()
    }

    /// Adds an element with the given classes as the last child of `parent`.
    pub(crate) fn add(&mut self, parent: ElementId, class_names: &[&str]) -> ElementId {
        let element = self.tree.create(Role::Element);
        self.add_classes(element, class_names);
        self.tree.append_child(parent, element);
        element
    }

    /// Adds a text as the last child of `parent`.
    pub(crate) fn add_text(&mut self, parent: ElementId, text: &str) -> ElementId {
        let element = self.tree.create(Role::Text);
        self.tree.set_text(element, text);
        self.tree.append_child(parent, element);
        element
    }

    pub(crate) fn add_classes(&mut self, element: ElementId, class_names: &[&str]) {
        for name in class_names {
            let class = self
                .style_sheet
                .class_named(name)
                .unwrap_or_else(|| panic!("the stylesheet names the class `{name}`"));
            self.tree.add_class(element, class);
        }
    }

    pub(crate) fn theme(&self, color_scheme: ColorScheme) -> Theme {
        match color_scheme {
            ColorScheme::Light => self.tokens.light.clone(),
            ColorScheme::Dark => self.tokens.dark.clone(),
        }
    }

    /// Runs the three stages of a frame.
    pub(crate) fn frame(&mut self) {
        self.tree.resolve_styles(self.time);
        self.tree
            .compute_layout(self.viewport, self.scale, &mut self.text_shaper);
        self.tree.build_display_list();
    }

    pub(crate) fn border_box(&self, element: ElementId) -> Rect {
        self.tree.border_box(element)
    }

    /// Runs a frame and draws its display list offscreen with `scale` device pixels to a CSS pixel.
    pub(crate) fn render(&mut self, renderer: &Renderer, scale: f32) -> FrameImage {
        self.scale = scale;
        self.frame();
        OffscreenTarget::new(renderer, self.viewport, scale).render(self.tree.display_list())
    }
}

/// A scene shown in a window; it remembers what the window asked of it.
pub(crate) struct WindowScene {
    pub(crate) scene: Scene,
    /// The viewport of every frame the window asked for.
    pub(crate) frame_viewports: Vec<Size>,
    /// Every color scheme the window switched to after it opened.
    pub(crate) color_scheme_changes: Vec<ColorScheme>,
}

impl WindowScene {
    /// A scene under the settings of the system whose tree asks `frame_scheduler` for frames.
    pub(crate) fn new(rules: &str, frame_scheduler: Box<dyn FrameScheduler>, settings: SurfaceSettings) -> Self {
        let mut scene = Scene::new(rules, Size::default());
        scene.tree.set_theme(scene.theme(settings.color_scheme));
        scene.tree.set_reduced_motion(settings.reduces_motion);
        scene.tree.set_frame_scheduler(frame_scheduler);
        Self {
            scene,
            frame_viewports: Vec::new(),
            color_scheme_changes: Vec::new(),
        }
    }
}

impl SurfaceContent for WindowScene {
    fn frame(&mut self, viewport: Size, scale: f32, now: Duration, show: &mut dyn FnMut(&ElementTree)) {
        self.frame_viewports.push(viewport);
        self.scene.viewport = viewport;
        self.scene.scale = scale;
        self.scene.time = now;
        self.scene.frame();
        show(&self.scene.tree);
    }

    fn color_scheme_changed(&mut self, color_scheme: ColorScheme) {
        self.color_scheme_changes.push(color_scheme);
        self.scene.tree.set_theme(self.scene.theme(color_scheme));
    }

    fn reduced_motion_changed(&mut self, reduces_motion: bool) {
        self.scene.tree.set_reduced_motion(reduces_motion);
    }

    /// The scene is changed by its test alone: a pointer that happens to be over the window
    /// changes nothing.
    fn pointer_input(&mut self, _input: &PointerInput) {}

    fn scroll_input(&mut self, _input: &ScrollInput) {}

    fn key_input(&mut self, _event: &KeyEvent) -> bool {
        false
    }

    fn accessibility_tree(&mut self) -> TreeUpdate {
        self.scene.tree.accessibility_tree()
    }

    fn accessibility_update(&mut self) -> TreeUpdate {
        self.scene.tree.accessibility_update()
    }

    fn accessibility_action(&mut self, _request: &ActionRequest) {}
}

/// Lets the main run loop deliver what is due, display-link callbacks among it, for `duration`.
pub(crate) fn run_main_loop(duration: Duration) {
    NSRunLoop::currentRunLoop().runUntilDate(&NSDate::dateWithTimeIntervalSinceNow(duration.as_secs_f64()));
}

/// What a test that runs on the main thread expected and did not see. Such a test checks
/// everything it can in one run of the application and reports at the end.
#[derive(Debug, Default)]
pub(crate) struct Expectations {
    failures: Vec<String>,
}

impl Expectations {
    pub(crate) fn expect(&mut self, holds: bool, expectation: &str, observed: &str) {
        if !holds {
            self.failures.push(format!("{expectation}: {observed}"));
        }
    }

    pub(crate) fn report(self, test_name: &str) -> ExitCode {
        if self.failures.is_empty() {
            println!("{test_name}: every expectation holds");
            ExitCode::SUCCESS
        } else {
            eprintln!("{test_name} failed:\n{}", self.failures.join("\n"));
            ExitCode::FAILURE
        }
    }
}
