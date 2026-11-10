use morok_platform::{Application, Renderer};

use crate::bench_error::BenchError;
use crate::budgets::Budgets;
use crate::reference_scene::ReferenceScene;
use crate::verdict::Verdict;
use crate::{frame_scenario, idle_scenario};

const FRAME: &str = "frame";
const IDLE: &str = "idle";

/// The scenarios the harness runs by name.
pub const NAMES: [&str; 2] = [FRAME, IDLE];

/// Runs the scenario of one of [`NAMES`] against the budgets.
///
/// # Panics
///
/// When the name is none of them: the command line is checked against the names first.
pub fn run(name: &str, budgets: &Budgets) -> Result<Verdict, BenchError> {
    match name {
        FRAME => frame(budgets),
        IDLE => idle(budgets),
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
