use std::rc::Rc;

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
}
