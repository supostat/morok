use std::cell::Cell;

use objc2::rc::Retained;
use objc2_foundation::{NSRunLoop, NSRunLoopCommonModes};
use objc2_quartz_core::CADisplayLink;

/// The display link of a surface. It ticks only while a frame is wanted: a request starts it,
/// and a frame during which no new request arrived stops it.
#[derive(Debug)]
pub(crate) struct FrameLoop {
    display_link: Retained<CADisplayLink>,
    /// Whether a frame was asked for since the last one began.
    frame_is_requested: Cell<bool>,
}

impl FrameLoop {
    /// Takes over a display link that is not yet scheduled. The link is added in the common
    /// modes, so frames keep coming while a window is resized or a menu is tracked.
    pub(crate) fn new(display_link: Retained<CADisplayLink>) -> Self {
        display_link.setPaused(true);
        // SAFETY: the mode is a constant of Foundation and the run loop is that of the main
        // thread, where the view the link belongs to lives.
        unsafe { display_link.addToRunLoop_forMode(&NSRunLoop::mainRunLoop(), NSRunLoopCommonModes) };
        Self {
            display_link,
            frame_is_requested: Cell::new(false),
        }
    }

    pub(crate) fn request_frame(&self) {
        self.frame_is_requested.set(true);
        self.display_link.setPaused(false);
    }

    /// From here on a request is for the frame after the one that begins.
    pub(crate) fn begin_frame(&self) {
        self.frame_is_requested.set(false);
    }

    pub(crate) fn end_frame(&self) {
        if !self.frame_is_requested.get() {
            self.display_link.setPaused(true);
        }
    }

    /// Ends the loop for good; the link lets go of the view it calls.
    pub(crate) fn stop(&self) {
        self.display_link.invalidate();
    }
}
