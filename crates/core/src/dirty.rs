/// The frame stages that still owe work to one element.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Dirty(u8);

impl Dirty {
    /// The style of the element must be resolved.
    pub(crate) const STYLE: Self = Self(1);
    /// A descendant carries [`Self::STYLE`].
    pub(crate) const STYLE_BELOW: Self = Self(1 << 1);
    /// Layout wrote a box for the element; its place on the surface must be worked out.
    pub(crate) const GEOMETRY: Self = Self(1 << 2);
    /// A descendant carries [`Self::GEOMETRY`].
    pub(crate) const GEOMETRY_BELOW: Self = Self(1 << 3);
    /// The display items of the element must be rebuilt.
    pub(crate) const PAINT: Self = Self(1 << 4);

    pub(crate) const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub(crate) const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub(crate) const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub(crate) fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub(crate) fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }
}
