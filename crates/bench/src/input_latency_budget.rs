use serde::Deserialize;

/// How late the glyph of a keypress may be shown, and how keypresses are sampled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputLatencyBudget {
    /// The 95th percentile of the frames presented after a keypress before the first that
    /// shows its glyph.
    pub late_frames: usize,
    /// Keypresses made before the measurement begins.
    pub warm_up_keypresses: usize,
    pub measured_keypresses: usize,
}
