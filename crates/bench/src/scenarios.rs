use morok_platform::{Application, Renderer};

use crate::bench_error::BenchError;
use crate::budgets::Budgets;
use crate::reference_scene::ReferenceScene;
use crate::verdict::Verdict;
use crate::{cold_start_scenario, frame_scenario, idle_scenario, input_latency_scenario, scroll_scenario};

const FRAME: &str = "frame";
const IDLE: &str = "idle";
const COLD_START: &str = "cold-start";
const INPUT_LATENCY: &str = "input-latency";
const SCROLL_1M: &str = "scroll-1m";

/// The scenarios the harness runs by name.
pub const NAMES: [&str; 5] = [FRAME, IDLE, COLD_START, INPUT_LATENCY, SCROLL_1M];

/// Runs the scenario of one of [`NAMES`] against the budgets.
///
/// # Panics
///
/// When the name is none of them: the command line is checked against the names first.
pub fn run(name: &str, budgets: &Budgets) -> Result<Verdict, BenchError> {
    match name {
        FRAME => frame(budgets),
        IDLE => idle(budgets),
        COLD_START => cold_start_scenario::run(COLD_START, &budgets.cold_start),
        INPUT_LATENCY => input_latency_scenario::run(INPUT_LATENCY, &budgets.input_latency),
        SCROLL_1M => Ok(scroll_scenario::run(SCROLL_1M, &Renderer::new()?, budgets)),
        _ => panic!("no scenario is named `{name}`"),
    }
}

fn frame(budgets: &Budgets) -> Result<Verdict, BenchError> {
    let renderer = Renderer::new()?;
    frame_scenario::run(FRAME, &mut ReferenceScene::changing(), &renderer, budgets)
}

fn idle(budgets: &Budgets) -> Result<Verdict, BenchError> {
    let application = Application::new()?;
    idle_scenario::run(
        IDLE,
        Box::new(ReferenceScene::settling()),
        &application,
        budgets.idle.duration(),
        &budgets.idle,
    )
}
