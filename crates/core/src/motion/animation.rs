use std::time::Duration;

use morok_style::Keyframes;

use crate::motion::animation_timing::AnimationTiming;
use crate::motion::iteration::Iteration;
use crate::motion::timeline::Timeline;

/// A keyframe animation an element runs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Animation {
    pub(crate) keyframes: Keyframes,
    pub(crate) start_time: Duration,
    pub(crate) timing: AnimationTiming,
}

impl Animation {
    pub(crate) fn is_running(&self, timeline: &Timeline) -> bool {
        self.timing.is_running(timeline.milliseconds_since(self.start_time))
    }

    /// Where the animation stands now; none when it shows nothing.
    pub(crate) fn iteration_now(&self, timeline: &Timeline) -> Option<Iteration> {
        self.timing.iteration_at(timeline.milliseconds_since(self.start_time))
    }
}
