/// What an assistive technology may ask an element to do, besides taking focus: an element
/// that takes focus offers that by itself.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AccessibleActions(u8);

impl AccessibleActions {
    pub const NONE: Self = Self(0);
    /// The default action of the element: what a click does.
    pub const CLICK: Self = Self(1);
    pub const INCREMENT: Self = Self(1 << 1);
    pub const DECREMENT: Self = Self(1 << 2);
    pub const SET_VALUE: Self = Self(1 << 3);

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}
