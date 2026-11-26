/// A set of element states; each state is a CSS pseudo-class a selector can require.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct States(u8);

impl States {
    /// The set of no state.
    pub const NONE: Self = Self(0);
    /// The pointer is over the element: `:hover`.
    pub const HOVER: Self = Self(1);
    /// The element is being pressed: `:active`.
    pub const ACTIVE: Self = Self(1 << 1);
    /// The element has the keyboard focus: `:focus`.
    pub const FOCUS: Self = Self(1 << 2);
    /// The element has the focus and shows it, as after a move by the keyboard: `:focus-visible`.
    pub const FOCUS_VISIBLE: Self = Self(1 << 3);
    /// The element or one of its descendants has the focus: `:focus-within`.
    pub const FOCUS_WITHIN: Self = Self(1 << 4);
    /// The element takes no input: `:disabled`.
    pub const DISABLED: Self = Self(1 << 5);
    /// The element is on or chosen: `:checked`.
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

    /// The states either set holds.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// The states of `self` that `other` does not hold.
    #[must_use]
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Whether the set holds every state of `other`.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether the set holds no state.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// How many states the set holds.
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }
}
