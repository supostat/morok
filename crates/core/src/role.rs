/// What an element is; decides how it is laid out and what it may contain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    /// A box that lays out its children.
    Element,
    /// A run of text, measured and shaped through the [`TextShaper`](crate::TextShaper).
    Text,
    Image,
    Icon,
    /// A region the application draws itself.
    Canvas,
    /// A box that lays out its children and scrolls them.
    Scroll,
    /// One line of plain text the user edits. The line starts at the left of the content box
    /// and scrolls inside it to keep the caret in view; it holds no line break.
    TextInput,
    /// Plain text the user edits in lines that wrap at the width of the content box and
    /// scroll inside it.
    TextArea,
}

impl Role {
    /// Whether elements of this role lay out children.
    pub const fn is_container(self) -> bool {
        matches!(self, Self::Element | Self::Scroll)
    }

    /// Whether elements of this role hold a text the user edits.
    pub const fn is_editable_text(self) -> bool {
        matches!(self, Self::TextInput | Self::TextArea)
    }

    /// Whether elements of this role set text in lines: a text or an edited text.
    pub(crate) const fn sets_lines(self) -> bool {
        matches!(self, Self::Text | Self::TextInput | Self::TextArea)
    }
}
