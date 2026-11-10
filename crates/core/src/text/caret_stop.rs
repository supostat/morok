/// A place a caret may stand in one line of an edited text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretStop {
    /// The byte of the text the caret stands before.
    pub offset: u32,
    /// From the start of the line to the caret.
    pub x: f32,
}
