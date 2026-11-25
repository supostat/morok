use std::time::Duration;

use crate::stage::Stage;

/// What one frame cost: the CPU time of each stage and the heap allocations made in it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameSample {
    stage_times: [Duration; Stage::ALL.len()],
    stage_allocations: [u64; Stage::ALL.len()],
}

impl FrameSample {
    pub fn new(stage_times: [Duration; Stage::ALL.len()], stage_allocations: [u64; Stage::ALL.len()]) -> Self {
        Self {
            stage_times,
            stage_allocations,
        }
    }

    pub fn time_of(&self, stage: Stage) -> Duration {
        self.stage_times[stage.index()]
    }

    /// The CPU time of the whole frame.
    pub fn total(&self) -> Duration {
        self.stage_times.iter().sum()
    }

    pub fn allocations_of(&self, stage: Stage) -> u64 {
        self.stage_allocations[stage.index()]
    }

    /// The heap allocations of the whole frame.
    pub fn allocations(&self) -> u64 {
        self.stage_allocations.iter().sum()
    }
}
