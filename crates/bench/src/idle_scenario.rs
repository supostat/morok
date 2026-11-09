use std::time::Duration;

use morok_platform::Application;

use crate::bench_error::BenchError;
use crate::idle_budget::IdleBudget;
use crate::idle_gate;
use crate::idle_run;
use crate::scene::Scene;
use crate::verdict::Verdict;

/// Leaves a window on the scene without events for `idle` and holds what its frame loop did
/// against the idle budget. The duration is the budget's for the gate, and shorter where the
/// gate itself is under test.
pub fn run(
    scenario: &str,
    scene: Box<dyn Scene>,
    application: &Application,
    idle: Duration,
    budget: &IdleBudget,
) -> Result<Verdict, BenchError> {
    let observation = idle_run::observe(application, scene, idle)?;
    Ok(idle_gate::judge(scenario, &observation, budget))
}
