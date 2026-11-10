use morok_style::{AnimationDirection, Easing, FillMode};

use crate::motion::iteration::Iteration;

/// The timing of a keyframe animation with every token resolved; times in milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AnimationTiming {
    pub(crate) duration: f64,
    /// Negative starts the animation part way through.
    pub(crate) delay: f64,
    /// Infinite for an animation that never ends.
    pub(crate) iterations: f64,
    pub(crate) direction: AnimationDirection,
    pub(crate) fill_mode: FillMode,
    /// The easing of every interval between two keyframes.
    pub(crate) easing: Easing,
}

impl AnimationTiming {
    fn active_duration(&self) -> f64 {
        if self.duration <= 0.0 || self.iterations <= 0.0 {
            0.0
        } else {
            self.duration * self.iterations
        }
    }

    fn end_time(&self) -> f64 {
        (self.delay + self.active_duration()).max(0.0)
    }

    /// Whether the animation still changes, or will, `elapsed` milliseconds after it started.
    pub(crate) fn is_running(&self, elapsed: f64) -> bool {
        elapsed < self.end_time()
    }

    /// Where the animation stands `elapsed` milliseconds after it started; none when it shows
    /// nothing then: before its delay or after its end without a fill.
    pub(crate) fn iteration_at(&self, elapsed: f64) -> Option<Iteration> {
        let active_duration = self.active_duration();
        let before_boundary = self.delay.min(self.end_time()).max(0.0);
        let after_boundary = (self.delay + active_duration).min(self.end_time()).max(0.0);
        let (active_time, is_before) = if elapsed < before_boundary {
            if !matches!(self.fill_mode, FillMode::Backwards | FillMode::Both) {
                return None;
            }
            ((elapsed - self.delay).max(0.0), true)
        } else if elapsed >= after_boundary {
            if !matches!(self.fill_mode, FillMode::Forwards | FillMode::Both) {
                return None;
            }
            ((elapsed - self.delay).min(active_duration).max(0.0), false)
        } else {
            (elapsed - self.delay, false)
        };
        let is_after = !is_before && elapsed >= after_boundary;
        let overall = if self.duration <= 0.0 {
            if is_before { 0.0 } else { self.iterations }
        } else {
            active_time / self.duration
        };
        let mut simple = if overall.is_infinite() { 0.0 } else { overall.fract() };
        if simple <= 0.0 && !is_before && active_time >= active_duration && self.iterations > 0.0 {
            simple = 1.0;
        }
        let current_iteration = if is_after && self.iterations.is_infinite() {
            f64::INFINITY
        } else if simple >= 1.0 {
            overall.floor() - 1.0
        } else {
            overall.floor()
        };
        let is_odd_iteration = current_iteration.is_finite() && current_iteration.rem_euclid(2.0) >= 1.0;
        let runs_forwards = match self.direction {
            AnimationDirection::Normal => true,
            AnimationDirection::Reverse => false,
            AnimationDirection::Alternate => !is_odd_iteration,
            AnimationDirection::AlternateReverse => is_odd_iteration,
        };
        let progress = if runs_forwards { simple } else { 1.0 - simple };
        Some(Iteration {
            progress,
            is_before: (is_before && runs_forwards) || (is_after && !runs_forwards),
        })
    }
}
