use std::time::Duration;

use crate::frame_sample::FrameSample;
use crate::percentile::percentile_95;
use crate::stage::Stage;

/// The samples of the measured frames of a scenario; the frames of the warm-up are not among them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameMeasurements {
    samples: Vec<FrameSample>,
}

impl FrameMeasurements {
    /// # Panics
    ///
    /// When no frame was measured: a percentile of nothing names no value.
    pub fn new(samples: Vec<FrameSample>) -> Self {
        assert!(!samples.is_empty(), "a measurement holds at least one frame");
        Self { samples }
    }

    pub fn frame_count(&self) -> usize {
        self.samples.len()
    }

    /// The time the stage took in 95 % of the frames or less.
    pub fn percentile_95(&self, stage: Stage) -> Duration {
        percentile_95(self.samples.iter().map(|sample| sample.time_of(stage)).collect())
    }

    /// The time the whole frame took in 95 % of the frames or less.
    pub fn total_percentile_95(&self) -> Duration {
        percentile_95(self.samples.iter().map(FrameSample::total).collect())
    }

    /// The measured frame that made the most heap allocations; the first of them on a tie.
    pub fn most_allocating_frame(&self) -> &FrameSample {
        self.samples
            .iter()
            .rev()
            .max_by_key(|sample| sample.allocations())
            .expect("a measurement holds at least one frame")
    }
}
