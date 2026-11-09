/// An icon rasterized at one size: coverage alone for a single-color icon, premultiplied blue,
/// green, red and alpha for one with colors of its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IconRaster {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) is_single_color: bool,
    pub(crate) pixels: Vec<u8>,
}
