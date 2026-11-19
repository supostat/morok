use std::fmt::{Debug, Formatter, Result};

use morok_core::ElementId;

use crate::view::action::Action;

/// What a timer of a surface does when it falls due.
pub(crate) struct TimerHandler {
    /// The element the timer goes with.
    pub(crate) owner: ElementId,
    pub(crate) run: Action,
}

impl Debug for TimerHandler {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("TimerHandler")
            .field("owner", &self.owner)
            .finish_non_exhaustive()
    }
}
