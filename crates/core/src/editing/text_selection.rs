use std::ops::Range;

/// The selected part of an edited text, in bytes of the text. The selection runs from the
/// anchor, where it was begun, to the head, which the keyboard and the pointer move; with both
/// at one place it is a caret.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextSelection {
    pub anchor: usize,
    pub head: usize,
}

impl TextSelection {
    /// A caret before the byte at `offset`.
    pub const fn caret(offset: usize) -> Self {
        Self {
            anchor: offset,
            head: offset,
        }
    }

    pub const fn is_caret(&self) -> bool {
        self.anchor == self.head
    }

    /// The selected bytes, whichever way the selection was drawn.
    pub fn range(&self) -> Range<usize> {
        self.anchor.min(self.head)..self.anchor.max(self.head)
    }
}
