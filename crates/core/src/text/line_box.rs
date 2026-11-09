use crate::geometry::rect::Rect;

/// The box of one line of a text element, as CSS lays it out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineBox {
    /// The line box, relative to the border box of the element: as high as the line height,
    /// as wide as the text of the line and placed by `text-align`.
    pub rect: Rect,
    /// From the top of the border box of the element down to the baseline of the line.
    pub baseline: f32,
}
