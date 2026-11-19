use std::time::Duration;

use crate::element_id::ElementId;
use crate::timing::timer_id::TimerId;

/// A timer that has not fallen due.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PendingTimer {
    pub(crate) id: TimerId,
    /// The element the timer goes with.
    pub(crate) owner: ElementId,
    pub(crate) delay: Duration,
    /// The time it falls due at; none until the first frame after it was started, whose time
    /// its delay counts from.
    pub(crate) due_at: Option<Duration>,
}
