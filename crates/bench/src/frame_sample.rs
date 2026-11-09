use std::time::Duration;

use crate::stage::Stage;

/// What one frame cost: the CPU time of each stage and the heap allocations it made.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameSample {
    stage_times: [Duration; Stage::ALL.len()],
    allocations: u64,
}

impl FrameSample {
    pub fn new(stage_times: [Duration; Stage::ALL.len()], allocations: u64) -> Self {
        Self {
            stage_times,
            allocations,
        }
    }

    pub fn time_of(&self, stage: Stage) -> Duration {
        self.stage_times[stage.index()]
    }

    /// The CPU time of the whole frame.
    pub fn total(&self) -> Duration {
        self.stage_times.iter().sum()
    }

    pub fn allocations(&self) -> u64 {
        self.allocations
    }
}
