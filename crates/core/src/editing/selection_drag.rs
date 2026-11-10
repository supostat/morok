use std::ops::Range;

/// A selection being drawn with the pointer held down: what the press selected stays
/// selected, and the selection grows from it to where the pointer is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SelectionDrag {
    /// What the press selected: a place, a word or a paragraph.
    pub(crate) origin: Range<usize>,
    pub(crate) unit: DragUnit,
}

/// What a drag selects by: set by the number of presses that began it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DragUnit {
    Character,
    Word,
    Paragraph,
}

impl DragUnit {
    pub(crate) const fn of_click_count(click_count: u32) -> Self {
        match click_count {
            0 | 1 => Self::Character,
            2 => Self::Word,
            _ => Self::Paragraph,
        }
    }
}
