use std::time::Duration;

use morok_style::AnimatableProperty;

use crate::motion::held_value::HeldValue;
use crate::motion::interpolation::interpolate;
use crate::motion::timeline::Timeline;
use crate::motion::transition_timing::TransitionTiming;

/// A running transition of one property of an element.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Transition {
    pub(crate) property: AnimatableProperty,
    pub(crate) start_value: HeldValue,
    pub(crate) end_value: HeldValue,
    /// What a transition back from this one has to reach to count as a reversal: the start
    /// value of the transition this one reversed, or its own.
    pub(crate) reversing_adjusted_start_value: HeldValue,
    /// How much of its full duration a reversal takes, from `0.0` to `1.0`.
    pub(crate) reversing_shortening_factor: f64,
    pub(crate) start_time: Duration,
    pub(crate) timing: TransitionTiming,
}

impl Transition {
    pub(crate) fn is_finished(&self, timeline: &Timeline) -> bool {
        self.timing.is_finished(timeline.milliseconds_since(self.start_time))
    }

    pub(crate) fn output_progress(&self, timeline: &Timeline) -> f64 {
        self.timing
            .output_progress(timeline.milliseconds_since(self.start_time))
    }

    /// Writes into `value` the value the transition shows now.
    pub(crate) fn value_now(&self, timeline: &Timeline, value: &mut HeldValue) {
        interpolate(
            self.property,
            self.start_value.value(),
            self.end_value.value(),
            self.output_progress(timeline),
            value,
        );
    }
}
