/// One glyph of a run and where it is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapedGlyph {
    /// The index of the glyph in its font.
    pub glyph: u16,
    /// From the start of the line to the origin of the glyph.
    pub x: f32,
    /// From the baseline down to the origin of the glyph.
    pub y: f32,
}
