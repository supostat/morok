use std::fmt::Debug;
use std::time::Duration;

/// The port through which a tree asks the host of its surface for a frame. The tree knows
/// nothing of display links: the platform implements the port with the frame loop of a window.
pub trait FrameScheduler: Debug {
    /// Asks for the stages of a frame to run soon.
    fn request_frame(&self);

    /// Asks for the stages of a frame to run once `delay` has passed, in place of the frame
    /// that was asked for this way before and has not run: what a tree shows may change with
    /// time alone, as a caret that blinks does.
    fn request_frame_after(&self, delay: Duration);
}
