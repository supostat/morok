/// The pixels of an image as the GPU takes them: rows from the top, four bytes of premultiplied
/// blue, green, red and alpha each.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecodedImage {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Vec<u8>,
}

impl DecodedImage {
    pub(crate) fn bytes(&self) -> usize {
        self.pixels.len()
    }
}
