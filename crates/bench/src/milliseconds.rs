use std::fmt::{Display, Formatter, Result};
use std::time::Duration;

use serde::Deserialize;

const MILLISECONDS_PER_SECOND: f64 = 1000.0;

/// A span of time as the budgets file and the reports write it.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Deserialize)]
#[serde(transparent)]
pub struct Milliseconds(pub f64);

impl Milliseconds {
    pub fn of(duration: Duration) -> Self {
        Self(duration.as_secs_f64() * MILLISECONDS_PER_SECOND)
    }

    pub fn duration(self) -> Duration {
        Duration::from_secs_f64(self.0 / MILLISECONDS_PER_SECOND)
    }
}

impl Display for Milliseconds {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{:.2} ms", self.0)
    }
}
