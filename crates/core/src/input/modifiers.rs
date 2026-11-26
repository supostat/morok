/// The modifier keys held while a key or a pointer button was used.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers(u8);

impl Modifiers {
    /// The set of no modifier key.
    pub const NONE: Self = Self(0);
    /// The Shift key.
    pub const SHIFT: Self = Self(1);
    /// The Control key.
    pub const CONTROL: Self = Self(1 << 1);
    /// The Option key.
    pub const OPTION: Self = Self(1 << 2);
    /// The Command key.
    pub const COMMAND: Self = Self(1 << 3);

    /// The modifiers either set holds.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Whether the set holds every modifier of `other`.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether the set holds any modifier of `other`.
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Whether the set holds no modifier.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
