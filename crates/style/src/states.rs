/// A set of element states; each state is a CSS pseudo-class a selector can require.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct States(u8);

impl States {
    pub const NONE: Self = Self(0);
    pub const HOVER: Self = Self(1);
    pub const ACTIVE: Self = Self(1 << 1);
    pub const FOCUS: Self = Self(1 << 2);
    pub const FOCUS_VISIBLE: Self = Self(1 << 3);
    pub const FOCUS_WITHIN: Self = Self(1 << 4);
    pub const DISABLED: Self = Self(1 << 5);
    pub const CHECKED: Self = Self(1 << 6);

    /// Every state with the name of its CSS pseudo-class.
    pub const PSEUDO_CLASSES: [(&'static str, Self); 7] = [
        ("hover", Self::HOVER),
        ("active", Self::ACTIVE),
        ("focus", Self::FOCUS),
        ("focus-visible", Self::FOCUS_VISIBLE),
        ("focus-within", Self::FOCUS_WITHIN),
        ("disabled", Self::DISABLED),
        ("checked", Self::CHECKED),
    ];

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }
}
