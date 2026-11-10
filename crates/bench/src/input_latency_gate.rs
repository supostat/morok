use crate::check::Check;
use crate::input_latency_budget::InputLatencyBudget;
use crate::percentile::percentile_95;
use crate::verdict::Verdict;

const GLYPH: &str = "glyph of a keypress";

/// Holds the measured keypresses of a scenario against the input latency budget: each is the
/// number of frames presented after the keypress before the first that showed its glyph.
///
/// # Panics
///
/// When no keypress was measured.
pub(crate) fn judge(scenario: &str, late_frames: &[usize], budget: &InputLatencyBudget) -> Verdict {
    let late = percentile_95(late_frames.to_vec());
    let checks = vec![Check {
        subject: GLYPH,
        measured: format!("p95 {late} frames late"),
        budget: format!("{} frames late", budget.late_frames),
        holds: late <= budget.late_frames,
    }];
    let latest = late_frames.iter().max().expect("a keypress was measured");
    let notes = vec![
        format!(
            "{} keypresses measured after {} of warm-up",
            late_frames.len(),
            budget.warm_up_keypresses
        ),
        format!("the latest glyph came {latest} frames late"),
    ];
    Verdict::new(scenario, checks, notes)
}

#[cfg(test)]
mod tests {
    use super::judge;
    use crate::input_latency_budget::InputLatencyBudget;

    const BUDGET: InputLatencyBudget = InputLatencyBudget {
        late_frames: 0,
        warm_up_keypresses: 5,
        measured_keypresses: 40,
    };

    #[test]
    fn glyphs_in_the_frames_of_their_keypresses_hold_the_budget() {
        let verdict = judge("input-latency", &[0; 40], &BUDGET);

        assert_eq!(verdict.exit_code(), 0, "{verdict}");
    }

    #[test]
    fn glyphs_late_in_more_than_one_keypress_of_twenty_fail_and_name_the_scenario_and_the_glyph() {
        let mut late_frames = vec![0; 37];
        late_frames.extend([1, 1, 2]);

        let verdict = judge("input-latency", &late_frames, &BUDGET);

        assert_eq!(verdict.exit_code(), 1);
        let report = verdict.to_string();
        assert!(
            report
                .lines()
                .any(|line| line.starts_with("input-latency: glyph of a keypress") && line.ends_with("OVER BUDGET")),
            "{report}"
        );
    }

    #[test]
    fn one_late_glyph_in_twenty_keypresses_does_not_reach_the_percentile() {
        let mut late_frames = vec![0; 19];
        late_frames.push(3);

        let verdict = judge("input-latency", &late_frames, &BUDGET);

        assert_eq!(verdict.exit_code(), 0, "{verdict}");
    }
}
