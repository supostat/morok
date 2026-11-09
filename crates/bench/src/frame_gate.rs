use crate::budgets::Budgets;
use crate::check::Check;
use crate::frame_measurements::FrameMeasurements;
use crate::milliseconds::Milliseconds;
use crate::stage::Stage;
use crate::verdict::Verdict;

const TOTAL: &str = "total";
const ALLOCATIONS: &str = "allocations";

/// Holds the measured frames of a scenario against the stage, total and allocation budgets.
pub(crate) fn judge(scenario: &str, measurements: &FrameMeasurements, budgets: &Budgets) -> Verdict {
    let mut checks: Vec<Check> = Stage::ALL
        .into_iter()
        .map(|stage| {
            time_check(
                stage.name(),
                Milliseconds::of(measurements.percentile_95(stage)),
                budgets.stages.of(stage),
            )
        })
        .collect();
    checks.push(time_check(
        TOTAL,
        Milliseconds::of(measurements.total_percentile_95()),
        budgets.frame.total,
    ));
    let most_allocations = measurements.most_allocations();
    checks.push(Check {
        subject: ALLOCATIONS,
        measured: format!("most {most_allocations} per frame"),
        budget: budgets.frame.allocations.to_string(),
        holds: most_allocations <= budgets.frame.allocations,
    });
    let notes = vec![format!(
        "{} frames measured after {} of warm-up",
        measurements.frame_count(),
        budgets.frame.warm_up_frames
    )];
    Verdict::new(scenario, checks, notes)
}

fn time_check(subject: &'static str, percentile_95: Milliseconds, budget: Milliseconds) -> Check {
    Check {
        subject,
        measured: format!("p95 {percentile_95}"),
        budget: budget.to_string(),
        holds: percentile_95 <= budget,
    }
}
