use std::time::Duration;

use crate::check::Check;
use crate::cold_start_budget::ColdStartBudget;
use crate::milliseconds::Milliseconds;
use crate::percentile::percentile_95;
use crate::verdict::Verdict;

const FIRST_FRAME: &str = "first frame";

/// Holds the measured launches of a scenario against the cold start budget.
///
/// # Panics
///
/// When no launch was measured.
pub(crate) fn judge(scenario: &str, launches: &[Duration], budget: &ColdStartBudget) -> Verdict {
    let first_frame = Milliseconds::of(percentile_95(launches.to_vec()));
    let checks = vec![Check {
        subject: FIRST_FRAME,
        measured: format!("p95 {first_frame}"),
        budget: budget.first_frame.to_string(),
        holds: first_frame <= budget.first_frame,
    }];
    let fastest = launches.iter().min().expect("a launch was measured");
    let slowest = launches.iter().max().expect("a launch was measured");
    let notes = vec![
        format!(
            "{} launches measured after {} of warm-up",
            launches.len(),
            budget.warm_up_launches
        ),
        format!(
            "fastest {}, slowest {}",
            Milliseconds::of(*fastest),
            Milliseconds::of(*slowest)
        ),
    ];
    Verdict::new(scenario, checks, notes)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::judge;
    use crate::cold_start_budget::ColdStartBudget;
    use crate::milliseconds::Milliseconds;

    const BUDGET: ColdStartBudget = ColdStartBudget {
        first_frame: Milliseconds(150.0),
        warm_up_launches: 2,
        measured_launches: 20,
    };

    fn launches(milliseconds: &[u64]) -> Vec<Duration> {
        milliseconds
            .iter()
            .map(|launch| Duration::from_millis(*launch))
            .collect()
    }

    #[test]
    fn launches_at_or_under_the_budget_hold_it() {
        let verdict = judge("cold-start", &launches(&[90, 150, 120]), &BUDGET);

        assert_eq!(verdict.exit_code(), 0, "{verdict}");
    }

    #[test]
    fn a_percentile_above_the_budget_fails_and_names_the_scenario_and_the_first_frame() {
        let verdict = judge("cold-start", &launches(&[90, 151, 120]), &BUDGET);

        assert_eq!(verdict.exit_code(), 1);
        let report = verdict.to_string();
        assert!(
            report
                .lines()
                .any(|line| line.starts_with("cold-start: first frame") && line.ends_with("OVER BUDGET")),
            "{report}"
        );
    }

    #[test]
    fn one_slow_launch_in_twenty_does_not_reach_the_percentile() {
        let mut measured = vec![100; 19];
        measured.push(400);

        let verdict = judge("cold-start", &launches(&measured), &BUDGET);

        assert_eq!(verdict.exit_code(), 0, "{verdict}");
    }
}
