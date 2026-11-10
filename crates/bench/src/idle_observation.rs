use std::time::Duration;

/// What the frame loop of a window did while the window was left without events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdleObservation {
    /// The frames presented from the first one until the window was left alone.
    pub settling_frames: u64,
    pub duration: Duration,
    pub presented_frames: u64,
    pub display_link_callbacks: u64,
}
