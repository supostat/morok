use crate::paint::item_range::ItemRange;

/// One line of an edited text as a caret moves through it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretLine {
    /// The bytes of the text the line holds, the white space and the line break it ends with
    /// among them.
    pub text: ItemRange,
    /// The stops of the line in the stop buffer, left to right: one before every character of
    /// the line that is no line break, and one after the last of them.
    pub stops: ItemRange,
}
