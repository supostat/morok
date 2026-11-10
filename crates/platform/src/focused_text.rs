use std::ops::Range;

use morok_core::TextSelection;

/// The text the focused element of a surface edits, as the input methods of the system read
/// it. Places are bytes of the text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusedText {
    pub text: String,
    pub selection: TextSelection,
    /// The bytes an input method is composing.
    pub marked: Option<Range<usize>>,
}
