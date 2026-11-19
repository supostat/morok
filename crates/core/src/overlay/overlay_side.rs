/// The side of its anchor an overlay lies on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OverlaySide {
    Below,
    Above,
    Right,
    Left,
}

impl OverlaySide {
    /// The side an overlay flips to when this one has no room for it.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Below => Self::Above,
            Self::Above => Self::Below,
            Self::Right => Self::Left,
            Self::Left => Self::Right,
        }
    }

    /// Whether the overlay lies under or over its anchor, so that it is aligned along the
    /// horizontal edge of the anchor.
    pub(crate) const fn is_vertical(self) -> bool {
        matches!(self, Self::Below | Self::Above)
    }
}
