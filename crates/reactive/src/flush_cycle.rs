use std::error::Error;
use std::fmt;

use crate::effect::Effect;

/// A flush that would not end: an effect kept being woken by writes made in the same flush.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlushCycle {
    /// The effect that reached the limit of runs in one flush.
    pub effect: Effect,
}

impl fmt::Display for FlushCycle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} kept running in one flush: it reads a signal that it, or an effect it wakes, writes",
            self.effect
        )
    }
}

impl Error for FlushCycle {}
