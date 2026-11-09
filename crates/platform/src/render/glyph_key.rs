use morok_core::FontId;

/// What tells one rasterized glyph from another. The size of the glyph is that of its font;
/// the scale and the phase are how the screen and the pen position sample it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct GlyphKey {
    pub(crate) font: FontId,
    pub(crate) glyph: u16,
    /// Device pixels per CSS pixel, as the bits of the float.
    pub(crate) scale: u32,
    /// Which quarter of a device pixel the pen stands on.
    pub(crate) subpixel_phase: u8,
}
