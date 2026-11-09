use std::fmt::Debug;

/// The port through which a tree asks the host of its surface for a frame. The tree knows
/// nothing of display links: the platform implements the port with the frame loop of a window.
pub trait FrameScheduler: Debug {
    /// Asks for the stages of a frame to run soon.
    fn request_frame(&self);
}
