mod support;

use std::process::ExitCode;
use std::time::Duration;

use morok_core::Size;
use morok_platform::{Application, FrameStatistics, Window, WindowOptions};
use morok_style::States;
use support::{Expectations, WindowScene, run_main_loop};

const TOOLBAR: &str = ".app { display: flex; gap: 20px; padding: 20px; background-color: var(--color-paper); }\n\
    .button { width: 100px; height: 30px; background-color: var(--color-blue); }\n\
    .button:hover { background-color: var(--color-red); }";

/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// Long enough for a display link that should have stopped to show that it has not.
const IDLE: Duration = Duration::from_millis(1500);
/// Long enough for a requested frame to be drawn.
const ONE_FRAME: Duration = Duration::from_millis(500);

fn frames_since(earlier: FrameStatistics, later: FrameStatistics) -> (u64, u64) {
    (
        later.display_link_callbacks - earlier.display_link_callbacks,
        later.presented_frames - earlier.presented_frames,
    )
}

fn main() -> ExitCode {
    let application = Application::new().expect("this Mac runs Metal 4");
    let options = WindowOptions {
        title: String::from("morok: display link"),
        content_size: Size {
            width: 400.0,
            height: 300.0,
        },
    };
    let mut button = None;
    let window = Window::open(&application, &options, |frame_scheduler, color_scheme| {
        let mut content = WindowScene::new(TOOLBAR, frame_scheduler, color_scheme);
        let root = content.scene.root();
        content.scene.add_classes(root, &["app"]);
        button = Some(content.scene.add(root, &["button"]));
        content
    });
    let button = button.expect("the window built its content");
    let mut expectations = Expectations::default();

    run_main_loop(SETTLING);
    let opened = window.frame_statistics();
    expectations.expect(
        opened.presented_frames >= 1,
        "a window with content presents its first frame",
        &format!("{opened:?}"),
    );

    run_main_loop(IDLE);
    let idle = window.frame_statistics();
    expectations.expect(
        idle == opened,
        "a window with content and no damage receives no display-link callback",
        &format!("{opened:?} became {idle:?}"),
    );

    window.update(|content| content.scene.tree.set_state(button, States::HOVER, true));
    run_main_loop(ONE_FRAME);
    let damaged = window.frame_statistics();
    expectations.expect(
        frames_since(idle, damaged) == (1, 1),
        "damage starts the display link for exactly one presented frame",
        &format!("{idle:?} became {damaged:?}"),
    );

    run_main_loop(IDLE);
    let idle_again = window.frame_statistics();
    expectations.expect(
        idle_again == damaged,
        "a frame that leaves no damage stops the display link",
        &format!("{damaged:?} became {idle_again:?}"),
    );

    window.update(|content| {
        content.scene.tree.set_state(button, States::HOVER, false);
        content.scene.tree.set_state(button, States::HOVER, true);
        content.scene.tree.set_state(button, States::HOVER, false);
    });
    run_main_loop(ONE_FRAME);
    let coalesced = window.frame_statistics();
    expectations.expect(
        frames_since(idle_again, coalesced) == (1, 1),
        "changes made before a frame are drawn by one frame",
        &format!("{idle_again:?} became {coalesced:?}"),
    );

    window.update(|content| content.scene.tree.set_state(button, States::ACTIVE, true));
    run_main_loop(ONE_FRAME);
    let undamaged = window.frame_statistics();
    expectations.expect(
        frames_since(coalesced, undamaged) == (1, 0),
        "a change that damages nothing runs one frame and presents none",
        &format!("{coalesced:?} became {undamaged:?}"),
    );

    run_main_loop(IDLE);
    let finished = window.frame_statistics();
    expectations.expect(
        finished == undamaged,
        "the display link stays stopped after a frame without damage",
        &format!("{undamaged:?} became {finished:?}"),
    );

    expectations.report("display link idle")
}
