use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use dispatch2::{DispatchQueue, DispatchTime, MainThreadBound};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_foundation::{NSRunLoop, NSRunLoopCommonModes};
use objc2_quartz_core::{CACurrentMediaTime, CADisplayLink};

/// The display link of a surface. It ticks only while a frame is wanted: a request starts it,
/// and a frame during which no new request arrived stops it.
#[derive(Debug)]
pub(crate) struct FrameLoop {
    display_link: Retained<CADisplayLink>,
    /// Whether a frame was asked for since the last one began.
    frame_is_requested: Cell<bool>,
    /// The time a block on the main queue waits for to ask for a frame; none while no block
    /// waits. A block that comes to a time this no longer holds asks for nothing.
    awaited_time: Cell<Option<Duration>>,
    /// How many blocks the loop put on the main queue to wait for a time.
    queued_blocks: Cell<u64>,
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
            awaited_time: Cell::new(None),
            queued_blocks: Cell::new(0),
        }
    }

    /// Asks for a frame at `time`, on the clock of Core Animation, in place of a frame asked
    /// for that way before. The display link stays paused until then, so a tree that waits for
    /// time alone takes no frames meanwhile. A tree asks for the same time in every frame until
    /// it comes, and one block waits for it.
    pub(crate) fn request_frame_at(self: &Rc<Self>, time: Duration) {
        if self.awaited_time.get() == Some(time) {
            return;
        }
        let wait = time.saturating_sub(Duration::from_secs_f64(CACurrentMediaTime()));
        if wait.is_zero() {
            self.awaited_time.set(None);
            self.request_frame();
            return;
        }
        self.awaited_time.set(Some(time));
        self.queued_blocks.set(self.queued_blocks.get() + 1);
        let main_thread = MainThreadMarker::new().expect("a frame loop lives on the main thread");
        let frame_loop = MainThreadBound::new(Rc::downgrade(self), main_thread);
        let when = DispatchTime::try_from(wait).unwrap_or(DispatchTime::FOREVER);
        DispatchQueue::main()
            .after(when, move || {
                let main_thread = MainThreadMarker::new().expect("the main queue runs on the main thread");
                if let Some(frame_loop) = frame_loop.get(main_thread).upgrade()
                    && frame_loop.awaited_time.get() == Some(time)
                {
                    frame_loop.awaited_time.set(None);
                    frame_loop.request_frame();
                }
            })
            .expect("the main queue takes work for later");
    }

    /// How many blocks the loop put on the main queue to wait for a time since it began.
    pub(crate) fn queued_blocks(&self) -> u64 {
        self.queued_blocks.get()
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
