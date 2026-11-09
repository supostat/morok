/// What the frame loop of a window has done since the window opened.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameStatistics {
    /// How many times the display link called back.
    pub display_link_callbacks: u64,
    /// How many frames were drawn and handed to the screen.
    pub presented_frames: u64,
}
