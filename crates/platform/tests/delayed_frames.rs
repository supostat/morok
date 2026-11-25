mod support;

use std::process::ExitCode;
use std::time::{Duration, Instant};

use morok_core::{ElementId, Focusability, Role, Size};
use morok_platform::{Application, FrameStatistics, Window, WindowOptions};
use support::{Expectations, WindowScene, run_main_loop};

const PAGE: &str = "@keyframes pulse { from { opacity: 0.2; } to { opacity: 1; } }\n\
    .app { display: flex; gap: 20px; padding: 20px; background-color: var(--color-paper); }\n\
    .field { width: 200px; height: 30px; color: var(--color-ink); }\n\
    .runner { width: 20px; height: 20px; background-color: var(--color-blue); \
    animation: pulse 1s linear infinite; }";

/// How long the caret of a field is shown, and then how long it is hidden.
const CARET_HALF_PERIOD: Duration = Duration::from_millis(530);
/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// Long enough for a change to be drawn by the frames it asks for.
const ONE_CHANGE: Duration = Duration::from_millis(500);
/// Long enough for the caret to turn several times.
const STRETCH: Duration = Duration::from_millis(3000);
/// How many times the turns of the caret the callbacks of a display link that an animation
/// keeps running at least number over a stretch.
const CALLBACKS_PER_TURN_WHILE_ANIMATING: u64 = 5;
/// How many callbacks a frame the caret asks for may take: the frame that draws the turn, and
/// one more if the drawable was not ready.
const CALLBACKS_PER_TURN_OF_THE_CARET_ALONE: u64 = 2;

/// A window on a focused field whose caret blinks beside an element that an endless animation
/// moves.
struct Stage {
    window: Window<WindowScene>,
    field: ElementId,
    runner: ElementId,
    expectations: Expectations,
}

/// What the window did over a stretch of time.
struct Stretch {
    before: FrameStatistics,
    after: FrameStatistics,
    elapsed: Duration,
}

impl Stretch {
    fn callbacks(&self) -> u64 {
        self.after.display_link_callbacks - self.before.display_link_callbacks
    }

    fn presented_frames(&self) -> u64 {
        self.after.presented_frames - self.before.presented_frames
    }

    fn queued_blocks(&self) -> u64 {
        self.after.queued_blocks - self.before.queued_blocks
    }

    /// The turns of the caret the stretch surely held.
    fn fewest_turns(&self) -> u64 {
        whole_half_periods(self.elapsed)
    }

    /// The turns of the caret the stretch may have held: a turn may fall at either end.
    fn most_turns(&self) -> u64 {
        whole_half_periods(self.elapsed) + 1
    }

    fn describe(&self) -> String {
        format!(
            "{} callbacks, {} presented frames, {} queued blocks over {:?}",
            self.callbacks(),
            self.presented_frames(),
            self.queued_blocks(),
            self.elapsed
        )
    }
}

fn whole_half_periods(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos() / CARET_HALF_PERIOD.as_nanos()).expect("a stretch lasts seconds")
}

impl Stage {
    fn open(application: &Application) -> Self {
        let options = WindowOptions {
            title: String::from("morok: delayed frames"),
            content_size: Size {
                width: 400.0,
                height: 120.0,
            },
            blur_behind: false,
        };
        let mut elements = None;
        let window = Window::open(application, &options, |frame_scheduler, settings| {
            let mut content = WindowScene::new(PAGE, frame_scheduler, settings);
            let root = content.scene.root();
            content.scene.add_classes(root, &["app"]);
            let field = content.scene.tree.create(Role::TextInput);
            content.scene.add_classes(field, &["field"]);
            content.scene.tree.append_child(root, field);
            content.scene.tree.set_text(field, "Folder");
            content.scene.tree.set_focusability(field, Focusability::TabStop);
            assert!(content.scene.tree.focus(field), "the field takes focus");
            let runner = content.scene.add(root, &["runner"]);
            elements = Some((field, runner));
            content
        });
        let (field, runner) = elements.expect("the window built its content");
        Self {
            window,
            field,
            runner,
            expectations: Expectations::default(),
        }
    }

    fn stretch(&self, duration: Duration) -> Stretch {
        let before = self.window.frame_statistics();
        let started = Instant::now();
        run_main_loop(duration);
        Stretch {
            before,
            after: self.window.frame_statistics(),
            elapsed: started.elapsed(),
        }
    }
}

/// The animation runs a frame at every tick of the display link, and every such frame asks for
/// the turn of the caret again: the window waits for each turn with one block.
fn while_an_animation_runs_one_block_waits_for_each_turn_of_the_caret(stage: &mut Stage) {
    let stretch = stage.stretch(STRETCH);
    stage.expectations.expect(
        stretch.callbacks() >= CALLBACKS_PER_TURN_WHILE_ANIMATING * stretch.most_turns(),
        "the animation keeps the display link running, many times as often as the caret turns",
        &stretch.describe(),
    );
    stage.expectations.expect(
        stretch.queued_blocks() <= stretch.most_turns() + 1,
        "the frames before a turn of the caret queue one block for it between them",
        &stretch.describe(),
    );
}

/// With the animation gone, the caret alone wakes the window: the block of a turn asks for the
/// frame that draws it.
fn the_caret_alone_presents_a_frame_at_every_turn(stage: &mut Stage) {
    let runner = stage.runner;
    stage.window.update(|content| content.scene.tree.remove(runner));
    run_main_loop(ONE_CHANGE);

    let stretch = stage.stretch(STRETCH);
    stage.expectations.expect(
        stretch.presented_frames() >= stretch.fewest_turns(),
        "the caret alone still blinks: every turn presents a frame",
        &stretch.describe(),
    );
    stage.expectations.expect(
        stretch.callbacks() <= CALLBACKS_PER_TURN_OF_THE_CARET_ALONE * stretch.most_turns(),
        "between its turns the caret takes no frames",
        &stretch.describe(),
    );
}

/// A field without focus shows no caret, so its tree asks for no time; the block queued for the
/// last turn may still ask for one frame.
fn a_window_whose_field_loses_focus_falls_idle(stage: &mut Stage) {
    let field = stage.field;
    stage.window.update(|content| {
        content.scene.tree.blur();
        assert_ne!(content.scene.tree.focused_text_input(), Some(field));
    });
    run_main_loop(ONE_CHANGE);

    let stretch = stage.stretch(STRETCH);
    stage.expectations.expect(
        stretch.callbacks() <= 1 && stretch.queued_blocks() == 0,
        "once its field loses focus the window takes at most the frame the last turn asked for and queues nothing",
        &stretch.describe(),
    );
}

fn main() -> ExitCode {
    let application = Application::new().expect("this Mac runs Metal 4");
    let mut stage = Stage::open(&application);
    run_main_loop(SETTLING);

    while_an_animation_runs_one_block_waits_for_each_turn_of_the_caret(&mut stage);
    the_caret_alone_presents_a_frame_at_every_turn(&mut stage);
    a_window_whose_field_loses_focus_falls_idle(&mut stage);

    stage.expectations.report("delayed frames")
}
