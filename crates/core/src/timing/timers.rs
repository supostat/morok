use std::time::Duration;

use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::timing::pending_timer::PendingTimer;
use crate::timing::timer_id::TimerId;

/// The timers of a tree. A delay counts from the first frame after its timer was started: the
/// time between frames is not known to a tree.
#[derive(Debug, Default)]
pub(crate) struct Timers {
    pending: Vec<PendingTimer>,
    /// The serial number of the last timer started.
    last_timer: u64,
}

impl Timers {
    pub(crate) fn start(&mut self, owner: ElementId, delay: Duration) -> TimerId {
        self.last_timer += 1;
        let id = TimerId::new(self.last_timer);
        self.pending.push(PendingTimer {
            id,
            owner,
            delay,
            due_at: None,
        });
        id
    }

    /// Stops the timer; one that fell due already is gone and stays so.
    pub(crate) fn cancel(&mut self, timer: TimerId) {
        self.pending.retain(|pending| pending.id != timer);
    }

    /// Moves the timers that fall due by `now` to `due`, the earliest first.
    pub(crate) fn take_due(&mut self, now: Duration, due: &mut Vec<TimerId>) {
        self.pending.sort_by_key(|pending| pending.due_at);
        self.pending.retain(|pending| {
            let is_due = pending.due_at.is_some_and(|due_at| due_at <= now);
            if is_due {
                due.push(pending.id);
            }
            !is_due
        });
    }

    /// Counts the delays of the timers started since the last frame from `now`. Returns how
    /// long from `now` until the earliest timer falls due; none while no timer runs.
    pub(crate) fn arm(&mut self, now: Duration) -> Option<Duration> {
        self.pending
            .iter_mut()
            .map(|pending| {
                let due_at = *pending.due_at.get_or_insert(now + pending.delay);
                due_at.saturating_sub(now)
            })
            .min()
    }

    /// Drops the timers of the elements that were removed.
    pub(crate) fn forget_removed(&mut self, elements: &ElementArena) {
        self.pending.retain(|pending| elements.contains(pending.owner));
    }
}
