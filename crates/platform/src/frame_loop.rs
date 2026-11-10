use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use dispatch2::{DispatchQueue, DispatchTime, MainThreadBound};
use objc2::MainThreadMarker;
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
    /// Counts the frames asked for after a delay; a delay that runs out after another was
    /// asked for asks for nothing.
    delayed_requests: Cell<u64>,
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
            delayed_requests: Cell::new(0),
        }
    }

    /// Asks for a frame once `delay` has passed, in place of a frame asked for that way before.
    /// The display link stays paused until then, so a tree that waits for time alone takes no
    /// frames meanwhile.
    pub(crate) fn request_frame_after(self: &Rc<Self>, delay: Duration) {
        let request = self.delayed_requests.get() + 1;
        self.delayed_requests.set(request);
        let main_thread = MainThreadMarker::new().expect("a frame loop lives on the main thread");
        let frame_loop = MainThreadBound::new(Rc::downgrade(self), main_thread);
        let when = DispatchTime::try_from(delay).unwrap_or(DispatchTime::FOREVER);
        DispatchQueue::main()
            .after(when, move || {
                let main_thread = MainThreadMarker::new().expect("the main queue runs on the main thread");
                if let Some(frame_loop) = frame_loop.get(main_thread).upgrade()
                    && frame_loop.delayed_requests.get() == request
                {
                    frame_loop.request_frame();
                }
            })
            .expect("the main queue takes work for later");
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
