/// What an element is; decides how it is laid out and what it may contain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    /// A box that lays out its children.
    Element,
    /// A run of text, measured through the [`TextMeasurer`](crate::TextMeasurer).
    Text,
    Image,
    Icon,
    /// A region the application draws itself.
    Canvas,
    /// A box that lays out its children and scrolls them.
    Scroll,
}

impl Role {
    /// Whether elements of this role lay out children.
    pub const fn is_container(self) -> bool {
        matches!(self, Self::Element | Self::Scroll)
    }
}
