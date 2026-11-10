use crate::check::Check;
use crate::idle_budget::IdleBudget;
use crate::idle_observation::IdleObservation;
use crate::verdict::Verdict;

const PRESENTED_FRAMES: &str = "presented frames";

/// Holds what a window did while idle against the idle budget.
pub(crate) fn judge(scenario: &str, observation: &IdleObservation, budget: &IdleBudget) -> Verdict {
    let checks = vec![Check {
        subject: PRESENTED_FRAMES,
        measured: format!(
            "{} in {:.1} s",
            observation.presented_frames,
            observation.duration.as_secs_f64()
        ),
        budget: budget.presented_frames.to_string(),
        holds: observation.presented_frames <= budget.presented_frames,
    }];
    let notes = vec![
        format!(
            "frames presented while the window settled: {}",
            observation.settling_frames
        ),
        format!("display link callbacks: {}", observation.display_link_callbacks),
    ];
    Verdict::new(scenario, checks, notes)
}
