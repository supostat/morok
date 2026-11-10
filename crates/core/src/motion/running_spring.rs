use std::time::Duration;

use crate::motion::spring::Spring;
use crate::motion::spring_id::SpringId;
use crate::motion::spring_sample::SpringSample;
use crate::motion::timeline::Timeline;

/// A spring a tree runs: it holds the frames of the tree until it settles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RunningSpring {
    pub(crate) id: SpringId,
    pub(crate) spring: Spring,
    pub(crate) from: f32,
    pub(crate) to: f32,
    pub(crate) velocity: f32,
    /// The time of the first frame after the spring was started; none until that frame.
    pub(crate) start_time: Option<Duration>,
    /// Whether the spring was started under reduced motion and fades instead of moving.
    pub(crate) fades: bool,
    pub(crate) is_settled: bool,
}

impl RunningSpring {
    pub(crate) fn sample(&self, timeline: &Timeline) -> SpringSample {
        if self.is_settled {
            return SpringSample::at_rest(self.to);
        }
        let elapsed = self
            .start_time
            .map_or(Duration::ZERO, |start| timeline.time_since(start));
        if self.fades {
            Spring::fade(self.to, elapsed)
        } else {
            self.spring.sample(self.from, self.to, self.velocity, elapsed)
        }
    }
}
