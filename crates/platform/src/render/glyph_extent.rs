/// The pixels of a rasterized glyph around its pen position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphExtent {
    pub(crate) width: u32,
    pub(crate) height: u32,
    /// From the pen position right to the left edge of the pixels.
    pub(crate) left: i32,
    /// From the baseline up to the top edge of the pixels.
    pub(crate) top: i32,
}
