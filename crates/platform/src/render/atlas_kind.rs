/// The two atlases, split by what a texel holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AtlasKind {
    /// One channel of coverage, tinted when drawn: glyphs of ordinary fonts.
    Mask,
    /// Premultiplied colors: glyphs of color fonts.
    Color,
}
