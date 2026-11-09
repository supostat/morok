#![forbid(unsafe_code)]

use std::hint::black_box;
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

use morok_bench::{Budgets, Scene, SceneTree, Verdict, frame_scenario, idle_scenario};
use morok_core::{ElementId, ElementTree, Size};
use morok_css::SourceFile;
use morok_platform::Application;
use morok_style::States;

const STYLESHEET: &str = "\
    .stage { padding: var(--space-4); background-color: var(--color-bg); }\n\
    .box { width: 100px; height: 30px; background-color: var(--color-surface); border-radius: var(--radius-sm); }\n\
    .box:hover { background-color: var(--color-accent-soft); }";

const OVER_BUDGET: u8 = 1;
const WITHIN_BUDGET: u8 = 0;
const COULD_NOT_RUN: i32 = 2;

/// Longer than the budget of any stage.
const SLOW: Duration = Duration::from_millis(3);
/// Long enough for a display link that keeps ticking to present frames.
const SHORT_IDLE: Duration = Duration::from_millis(1500);

/// What reaches the tree of a case between frames.
#[derive(Clone, Copy, Debug)]
enum Changes {
    /// The hover state of the box flips: a small change that damages the box.
    FlipHover,
    /// The box flips and the frame is held up by `Duration` before its stages run.
    Spin(Duration),
    /// The first frame is held up; the others only flip the box.
    SpinOnFirstFrame(Duration),
    /// The box flips and the heap is asked for one allocation.
    Allocate,
}

/// One box on a stage; what happens to it between frames is the case's.
#[derive(Debug)]
struct Case {
    changes: Changes,
    stage_box: Option<ElementId>,
}

impl Case {
    fn new(changes: Changes) -> Self {
        Self {
            changes,
            stage_box: None,
        }
    }

    fn flip_hover(&self, tree: &mut ElementTree) {
        let stage_box = self.stage_box.expect("the case built its box");
        let is_hovered = tree.states(stage_box).contains(States::HOVER);
        tree.set_state(stage_box, States::HOVER, !is_hovered);
    }
}

impl Scene for Case {
    fn viewport(&self) -> Size {
        Size {
            width: 200.0,
            height: 100.0,
        }
    }

    fn stylesheet(&self) -> SourceFile {
        SourceFile::new("budget_gate.css", STYLESHEET)
    }

    fn build(&mut self, tree: &mut SceneTree) {
        let root = tree.root();
        tree.add_classes(root, &["stage"]);
        self.stage_box = Some(tree.add(root, &["box"]));
    }

    fn change(&mut self, frame: usize, tree: &mut ElementTree) {
        match self.changes {
            Changes::Spin(duration) => spin(duration),
            Changes::SpinOnFirstFrame(duration) if frame == 0 => spin(duration),
            Changes::Allocate => {
                black_box(Box::new(frame));
            }
            Changes::FlipHover | Changes::SpinOnFirstFrame(_) => {}
        }
        self.flip_hover(tree);
    }
}

/// Holds the thread busy for `duration`.
fn spin(duration: Duration) {
    let started = Instant::now();
    while started.elapsed() < duration {
        std::hint::spin_loop();
    }
}

fn reports_over_budget(verdict: &Verdict, scenario: &str, subject: &str) -> bool {
    let report = verdict.to_string();
    verdict.exit_code() == OVER_BUDGET
        && report
            .lines()
            .any(|line| line.starts_with(&format!("{scenario}: {subject}")) && line.ends_with("OVER BUDGET"))
}

fn main() -> ExitCode {
    let application = Application::new().expect("this Mac runs Metal 4");
    let budgets = Budgets::of_workspace().expect("the budgets of the workspace read");
    let renderer = application.renderer();
    let mut expectations = Expectations::default();

    let slowed = frame_scenario::run("slowed", &mut Case::new(Changes::Spin(SLOW)), renderer, &budgets)
        .expect("the slowed scene runs");
    expectations.expect(
        reports_over_budget(&slowed, "slowed", "events"),
        "a 95th percentile above the budget exits 1 and names the scenario and the stage",
        &slowed.to_string(),
    );

    let fast = frame_scenario::run("fast", &mut Case::new(Changes::FlipHover), renderer, &budgets)
        .expect("the fast scene runs");
    expectations.expect(
        fast.exit_code() == WITHIN_BUDGET,
        "a value at or under the budget exits 0",
        &fast.to_string(),
    );

    let allocating = frame_scenario::run("allocating", &mut Case::new(Changes::Allocate), renderer, &budgets)
        .expect("the allocating scene runs");
    expectations.expect(
        reports_over_budget(&allocating, "allocating", "allocations"),
        "an allocation in a steady frame fails",
        &allocating.to_string(),
    );

    let slow_start = frame_scenario::run(
        "slow-start",
        &mut Case::new(Changes::SpinOnFirstFrame(SLOW)),
        renderer,
        &budgets,
    )
    .expect("the scene with a slow first frame runs");
    expectations.expect(
        slow_start.exit_code() == WITHIN_BUDGET,
        "warm-up frames are not measured: a slow first frame does not fail the gate",
        &slow_start.to_string(),
    );

    let ticking = idle_scenario::run(
        "ticking",
        Box::new(Case::new(Changes::FlipHover)),
        &application,
        SHORT_IDLE,
        &budgets.idle,
    )
    .expect("the ticking scene opens in a window");
    expectations.expect(
        reports_over_budget(&ticking, "ticking", "presented frames"),
        "a frame during idle fails",
        &ticking.to_string(),
    );

    let unknown = Command::new(env!("CARGO_BIN_EXE_morok-bench"))
        .arg("nope")
        .output()
        .expect("the harness binary runs");
    expectations.expect(
        unknown.status.code() == Some(COULD_NOT_RUN) && String::from_utf8_lossy(&unknown.stderr).contains("nope"),
        "an unknown scenario name exits 2",
        &format!("{:?}: {}", unknown.status, String::from_utf8_lossy(&unknown.stderr)),
    );

    expectations.report("budget gate")
}

/// What a test that runs on the main thread expected and did not see. Such a test checks
/// everything it can in one run of the application and reports at the end.
#[derive(Debug, Default)]
struct Expectations {
    failures: Vec<String>,
}

impl Expectations {
    fn expect(&mut self, holds: bool, expectation: &str, observed: &str) {
        if !holds {
            self.failures.push(format!("{expectation}:\n{observed}"));
        }
    }

    fn report(self, test_name: &str) -> ExitCode {
        if self.failures.is_empty() {
            println!("{test_name}: every expectation holds");
            ExitCode::SUCCESS
        } else {
            eprintln!("{test_name} failed:\n{}", self.failures.join("\n"));
            ExitCode::FAILURE
        }
    }
}
