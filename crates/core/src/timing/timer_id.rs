/// The handle of a timer a tree runs; it names nothing once the timer fell due or was cancelled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimerId(u64);

impl TimerId {
    pub(crate) const fn new(serial: u64) -> Self {
        Self(serial)
    }
}
