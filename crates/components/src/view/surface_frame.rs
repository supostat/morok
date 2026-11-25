use morok_core::FrameOutcome;

/// How the frame of a surface went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceFrame {
    /// An effect closed the surface during the frame, so its tree ran no stage.
    Closed,
    /// The stages of the tree ran and did what the outcome counts.
    Ran(FrameOutcome),
}
