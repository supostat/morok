/// The handle of a spring a tree runs; it names nothing once the spring is stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpringId(u64);

impl SpringId {
    pub(crate) const fn new(serial: u64) -> Self {
        Self(serial)
    }
}
