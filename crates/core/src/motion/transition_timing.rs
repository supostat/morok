use morok_style::Easing;

use crate::motion::easing_curve::eased;

/// The timing of a transition with every token resolved; times in milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TransitionTiming {
    pub(crate) duration: f64,
    /// Negative starts the transition part way.
    pub(crate) delay: f64,
    pub(crate) easing: Easing,
}

impl TransitionTiming {
    /// Whether a transition with this timing shows anything: one that takes no time at all
    /// does not start.
    pub(crate) fn takes_time(&self) -> bool {
        self.duration.max(0.0) + self.delay > 0.0
    }

    pub(crate) fn is_finished(&self, elapsed: f64) -> bool {
        elapsed >= self.delay + self.duration
    }

    /// The eased progress `elapsed` milliseconds after the start; during the delay the
    /// transition holds its start.
    pub(crate) fn output_progress(&self, elapsed: f64) -> f64 {
        if elapsed < self.delay {
            return eased(&self.easing, 0.0, true);
        }
        let progress = if self.duration > 0.0 {
            ((elapsed - self.delay) / self.duration).min(1.0)
        } else {
            1.0
        };
        eased(&self.easing, progress, false)
    }
}
