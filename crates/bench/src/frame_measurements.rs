use std::time::Duration;

use crate::frame_sample::FrameSample;
use crate::stage::Stage;

const PERCENTILE: f64 = 0.95;

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
        nearest_rank(self.samples.iter().map(|sample| sample.time_of(stage)).collect())
    }

    /// The time the whole frame took in 95 % of the frames or less.
    pub fn total_percentile_95(&self) -> Duration {
        nearest_rank(self.samples.iter().map(FrameSample::total).collect())
    }

    /// The most heap allocations any measured frame made.
    pub fn most_allocations(&self) -> u64 {
        self.samples
            .iter()
            .map(FrameSample::allocations)
            .max()
            .expect("a measurement holds at least one frame")
    }
}

/// The value at the 95th percentile by the nearest-rank method: the smallest value that is at
/// least as large as 95 % of them.
fn nearest_rank(mut values: Vec<Duration>) -> Duration {
    values.sort_unstable();
    let rank = (PERCENTILE * values.len() as f64).ceil() as usize;
    values[rank.max(1) - 1]
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::nearest_rank;

    fn milliseconds(values: &[u64]) -> Vec<Duration> {
        values.iter().map(|value| Duration::from_millis(*value)).collect()
    }

    #[test]
    fn the_percentile_of_one_value_is_that_value() {
        assert_eq!(nearest_rank(milliseconds(&[7])), Duration::from_millis(7));
    }

    #[test]
    fn the_percentile_of_twenty_values_is_the_nineteenth_in_order() {
        let values: Vec<u64> = (1..=20).rev().collect();
        assert_eq!(nearest_rank(milliseconds(&values)), Duration::from_millis(19));
    }

    #[test]
    fn one_slow_frame_in_a_hundred_does_not_reach_the_percentile() {
        let mut values = vec![1; 99];
        values.push(50);
        assert_eq!(nearest_rank(milliseconds(&values)), Duration::from_millis(1));
    }

    #[test]
    fn six_slow_frames_in_a_hundred_do() {
        let mut values = vec![1; 94];
        values.extend([50; 6]);
        assert_eq!(nearest_rank(milliseconds(&values)), Duration::from_millis(50));
    }
}
