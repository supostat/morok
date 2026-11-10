/// An icon the components of the kit show. A host registers each with its renderer once, and
/// every surface opened after that shows them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum KitIcon {
    /// The magnifier of a search field.
    Search,
}

impl KitIcon {
    pub const ALL: [Self; 1] = [Self::Search];

    /// The SVG of the icon, as the design book draws it.
    pub const fn svg(self) -> &'static str {
        match self {
            Self::Search => include_str!("../../../design/icons/search.svg"),
        }
    }

    /// The place of the icon in [`Self::ALL`].
    pub(crate) const fn index(self) -> usize {
        self as usize
    }
}
