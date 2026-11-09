use serde::Deserialize;

use crate::milliseconds::Milliseconds;

/// What a whole frame may cost, and how the frames are sampled.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameBudget {
    /// The 95th percentile of the CPU time of a frame, all stages together.
    pub total: Milliseconds,
    /// Heap allocations a steady frame may make.
    pub allocations: u64,
    /// Frames run before the measurement begins.
    pub warm_up_frames: usize,
    pub measured_frames: usize,
}
