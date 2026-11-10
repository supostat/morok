use std::time::Duration;

/// The time the frames of a tree run at and how much motion they show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Timeline {
    /// The time of the frame being run, on the clock of the host.
    pub(crate) now: Duration,
    /// Whether the user asked the system to reduce motion: transitions end at once, springs fade
    /// and animations that never end do not run.
    pub(crate) reduces_motion: bool,
}

impl Timeline {
    /// The time from `start` to now.
    ///
    /// # Panics
    ///
    /// When `start` is later than now: a motion starts in a frame no later than the one it is
    /// shown in.
    pub(crate) fn time_since(&self, start: Duration) -> Duration {
        self.now
            .checked_sub(start)
            .expect("a motion starts no later than the frame it is shown in")
    }

    /// The milliseconds from `start` to now.
    pub(crate) fn milliseconds_since(&self, start: Duration) -> f64 {
        self.time_since(start).as_secs_f64() * MILLISECONDS_PER_SECOND
    }
}

const MILLISECONDS_PER_SECOND: f64 = 1000.0;
