use std::time::Duration;

use serde::Deserialize;

/// What a window left alone may do.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdleBudget {
    /// How long the window is left without events.
    pub seconds: f64,
    /// Frames that may be presented in that time.
    pub presented_frames: u64,
}

impl IdleBudget {
    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.seconds)
    }
}
