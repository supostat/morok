/// Where a command of the keyboard takes a caret from where it stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Movement {
    GraphemeBackward,
    GraphemeForward,
    /// To the start of the word the caret is in or after.
    WordBackward,
    /// To the end of the word the caret is in or before.
    WordForward,
    /// To the start of the line as it is laid out.
    LineStart,
    /// To the end of the line as it is laid out.
    LineEnd,
    LineUp,
    LineDown,
    /// To the start of the paragraph, staying there when already at it.
    ParagraphStart,
    /// To the end of the paragraph, staying there when already at it.
    ParagraphEnd,
    /// To the start of the paragraph, or of the paragraph before when already at it.
    ParagraphBackward,
    /// To the end of the paragraph, or of the paragraph after when already at it.
    ParagraphForward,
    /// Up by what the box shows of the text.
    PageUp,
    /// Down by what the box shows of the text.
    PageDown,
    DocumentStart,
    DocumentEnd,
}

impl Movement {
    /// Whether the movement goes towards the start of the text.
    pub(crate) const fn is_backward(self) -> bool {
        matches!(
            self,
            Self::GraphemeBackward
                | Self::WordBackward
                | Self::LineStart
                | Self::LineUp
                | Self::ParagraphStart
                | Self::ParagraphBackward
                | Self::PageUp
                | Self::DocumentStart
        )
    }

    /// Whether the movement goes from line to line and keeps the caret in its column.
    pub(crate) const fn is_vertical(self) -> bool {
        matches!(self, Self::LineUp | Self::LineDown | Self::PageUp | Self::PageDown)
    }
}
