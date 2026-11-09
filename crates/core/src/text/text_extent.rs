/// The size of a text in lines: the tree turns the line count into a height with the line
/// height of CSS.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextExtent {
    /// The width of the widest line, without the white space lines end with.
    pub width: f32,
    pub line_count: usize,
}
