/// A handle to a registered shortcut.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShortcutId(u64);

impl ShortcutId {
    pub(crate) const fn new(serial: u64) -> Self {
        Self(serial)
    }

    /// The number of the shortcut among those its tree registered: what names its line of
    /// the menu bar to the host that shows the bar.
    pub(crate) const fn serial(self) -> u64 {
        self.0
    }
}
