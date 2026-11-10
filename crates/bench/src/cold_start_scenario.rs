use crate::bench_error::BenchError;
use crate::cold_start_budget::ColdStartBudget;
use crate::cold_start_gate;
use crate::cold_start_launch;
use crate::verdict::Verdict;

/// Launches the application of the harness as often as the budget asks for, the launches of
/// the warm-up unmeasured, and holds the time each measured launch took to its first presented
/// frame against the cold start budget.
pub fn run(scenario: &str, budget: &ColdStartBudget) -> Result<Verdict, BenchError> {
    for _ in 0..budget.warm_up_launches {
        cold_start_launch::time_to_first_frame()?;
    }
    let launches = (0..budget.measured_launches)
        .map(|_| cold_start_launch::time_to_first_frame())
        .collect::<Result<Vec<_>, _>>()?;
    Ok(cold_start_gate::judge(scenario, &launches, budget))
}
