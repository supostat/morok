/// A handle to a registered shortcut.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShortcutId(u64);

impl ShortcutId {
    pub(crate) const fn new(serial: u64) -> Self {
        Self(serial)
    }
}
