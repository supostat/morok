use serde::Deserialize;

use crate::milliseconds::Milliseconds;

/// How long a process may take to show its first frame, and how its launches are sampled.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColdStartBudget {
    /// The 95th percentile of the time from the start of a process to its first presented
    /// frame with content.
    pub first_frame: Milliseconds,
    /// Launches made before the measurement begins.
    pub warm_up_launches: usize,
    pub measured_launches: usize,
}
