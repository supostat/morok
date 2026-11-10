use std::ops::Range;

use crate::editing::text_selection::TextSelection;

/// A text an input method is composing: it stands in the edited text, marked, until the
/// input method settles on it or gives it up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Composition {
    /// The marked bytes of the text.
    pub(crate) range: Range<usize>,
    /// What the composition replaced when it began.
    pub(crate) replaced: String,
    pub(crate) selection_before: TextSelection,
}
