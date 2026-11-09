/// How a pixel of a rendering stands against the same pixel of the etalon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PixelClass {
    /// Equal, or different by no more than a pixel may be.
    Matching,
    /// Different, but smoothing an edge: it is left out of the color gate.
    AntiAliasing,
    Mismatching,
}
