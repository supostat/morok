use crate::paint::item_range::ItemRange;

/// One line of a shaped text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapedLine {
    /// The advance of the line without the white space it ends with.
    pub width: f32,
    /// The runs of the line, left to right, in the run buffer it was shaped into.
    pub runs: ItemRange,
}
