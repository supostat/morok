use std::rc::Rc;
use std::time::Duration;

use morok_core::FrameScheduler;

use crate::frame_loop::FrameLoop;

/// The frame scheduler of the tree a window shows: a request starts the display link of the
/// window.
#[derive(Debug)]
pub(crate) struct FrameRequester {
    pub(crate) frame_loop: Rc<FrameLoop>,
}

impl FrameScheduler for FrameRequester {
    fn request_frame(&self) {
        self.frame_loop.request_frame();
    }

    /// The time is that of the display link of the window, which passes its target timestamp
    /// to the frames of the tree.
    fn request_frame_at(&self, time: Duration) {
        self.frame_loop.request_frame_at(time);
    }
}
