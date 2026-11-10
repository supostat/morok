mod support;

use std::process::ExitCode;
use std::time::Duration;

use morok_core::Size;
use morok_platform::{Appearance, Application, Window, WindowOptions};
use morok_style::ColorScheme;
use support::{Expectations, WindowScene, run_main_loop};

const PAGE: &str = ".page { background-color: var(--color-paper); }";

const OPENED: Size = Size {
    width: 400.0,
    height: 300.0,
};
const RESIZED: Size = Size {
    width: 520.0,
    height: 360.0,
};

/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// Long enough for the system to deliver a change and for the frame it asks for to be drawn.
const ONE_CHANGE: Duration = Duration::from_millis(700);

fn appearance_of(color_scheme: ColorScheme) -> Appearance {
    match color_scheme {
        ColorScheme::Light => Appearance::Light,
        ColorScheme::Dark => Appearance::Dark,
    }
}

fn the_first_frames_fill_the_opened_surface(window: &Window<WindowScene>, expectations: &mut Expectations) {
    run_main_loop(SETTLING);

    let viewports = window.update(|content| content.frame_viewports.clone());
    expectations.expect(
        window.viewport() == OPENED && viewports.last() == Some(&OPENED),
        "the content is laid out in the surface the window opened with",
        &format!("the window is {:?}, the frames were {viewports:?}", window.viewport()),
    );
    expectations.expect(
        window.frame_statistics().presented_frames >= 1,
        "an opened window presents its content",
        &format!("{:?}", window.frame_statistics()),
    );
}

fn a_resize_lays_the_content_out_again_and_redraws_it(window: &Window<WindowScene>, expectations: &mut Expectations) {
    let before = window.frame_statistics();

    window.set_content_size(RESIZED);
    run_main_loop(ONE_CHANGE);

    let last_viewport = window.update(|content| content.frame_viewports.last().copied());
    let after = window.frame_statistics();
    expectations.expect(
        window.viewport() == RESIZED && last_viewport == Some(RESIZED),
        "a resized window lays its content out in the new surface",
        &format!(
            "the window is {:?}, the last frame was {last_viewport:?}",
            window.viewport()
        ),
    );
    expectations.expect(
        after.presented_frames > before.presented_frames,
        "a resized surface is drawn again",
        &format!("{before:?} became {after:?}"),
    );
}

fn the_application_overrides_the_system_appearance_and_gives_it_back(
    application: &Application,
    window: &Window<WindowScene>,
    expectations: &mut Expectations,
) {
    let system_color_scheme = application.color_scheme();
    let other_color_scheme = match system_color_scheme {
        ColorScheme::Light => ColorScheme::Dark,
        ColorScheme::Dark => ColorScheme::Light,
    };
    let changes = |window: &Window<WindowScene>| window.update(|content| content.color_scheme_changes.clone());
    let before = window.frame_statistics();

    application.set_appearance(appearance_of(other_color_scheme));
    run_main_loop(ONE_CHANGE);
    let after = window.frame_statistics();
    expectations.expect(
        changes(window) == [other_color_scheme] && application.color_scheme() == other_color_scheme,
        "an appearance the application chooses overrides that of the system",
        &format!(
            "the content was told {:?}, the application shows {}",
            changes(window),
            application.color_scheme()
        ),
    );
    expectations.expect(
        after.presented_frames > before.presented_frames,
        "a surface in a new color scheme is drawn again",
        &format!("{before:?} became {after:?}"),
    );

    application.set_appearance(appearance_of(other_color_scheme));
    run_main_loop(ONE_CHANGE);
    expectations.expect(
        changes(window) == [other_color_scheme],
        "choosing the appearance already shown tells the content nothing",
        &format!("{:?}", changes(window)),
    );

    application.set_appearance(Appearance::System);
    run_main_loop(ONE_CHANGE);
    expectations.expect(
        changes(window) == [other_color_scheme, system_color_scheme]
            && application.color_scheme() == system_color_scheme,
        "an application without an appearance of its own follows the system again",
        &format!(
            "the content was told {:?}, the application shows {}",
            changes(window),
            application.color_scheme()
        ),
    );
}

fn main() -> ExitCode {
    let application = Application::new().expect("this Mac runs Metal 4");
    let options = WindowOptions {
        title: String::from("morok: window surface"),
        content_size: OPENED,
    };
    let mut opened_in = None;
    let window = Window::open(&application, &options, |frame_scheduler, settings| {
        opened_in = Some(settings.color_scheme);
        let mut content = WindowScene::new(PAGE, frame_scheduler, settings);
        let root = content.scene.root();
        content.scene.add_classes(root, &["page"]);
        content
    });
    let mut expectations = Expectations::default();
    expectations.expect(
        opened_in == Some(application.color_scheme()),
        "a window opens in the color scheme of the system",
        &format!("{opened_in:?}, the system shows {}", application.color_scheme()),
    );

    the_first_frames_fill_the_opened_surface(&window, &mut expectations);
    a_resize_lays_the_content_out_again_and_redraws_it(&window, &mut expectations);
    the_application_overrides_the_system_appearance_and_gives_it_back(&application, &window, &mut expectations);

    expectations.report("window surface")
}
