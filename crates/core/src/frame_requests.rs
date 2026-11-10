use std::time::Duration;

use crate::frame_scheduler::FrameScheduler;

/// The frames a tree asks its host for: one request serves every change made before the frame
/// begins.
#[derive(Debug)]
pub(crate) struct FrameRequests {
    scheduler: Option<Box<dyn FrameScheduler>>,
    /// Whether a change waits for a frame that has not begun.
    is_pending: bool,
}

impl FrameRequests {
    /// The requests of a new tree, which waits for its first frame.
    pub(crate) fn awaiting_first_frame() -> Self {
        Self {
            scheduler: None,
            is_pending: true,
        }
    }

    /// Names the host that runs the frames. A frame that is already waited for is asked of it
    /// at once.
    pub(crate) fn set_scheduler(&mut self, scheduler: Box<dyn FrameScheduler>) {
        if self.is_pending {
            scheduler.request_frame();
        }
        self.scheduler = Some(scheduler);
    }

    /// Asks the host for a frame unless a frame that has not begun already waits. A change made
    /// once the first stage ran is too late for the running frame and asks for the next one.
    pub(crate) fn request(&mut self) {
        if self.is_pending {
            return;
        }
        self.is_pending = true;
        if let Some(scheduler) = &self.scheduler {
            scheduler.request_frame();
        }
    }

    /// Asks the host for a frame once `delay` has passed, whether or not one waits already:
    /// the frame that waits runs before then.
    pub(crate) fn request_after(&self, delay: Duration) {
        if let Some(scheduler) = &self.scheduler {
            scheduler.request_frame_after(delay);
        }
    }

    pub(crate) fn frame_began(&mut self) {
        self.is_pending = false;
    }
}
