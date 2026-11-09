const CHANNELS: usize = 4;

/// A rendered frame: RGBA, 8 bits per channel, rows from the top. The color channels are sRGB
/// values premultiplied by alpha, exactly as the frame buffer holds them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameImage {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

impl FrameImage {
    /// The red, green, blue and alpha of the pixel in column `x` of row `y`.
    ///
    /// # Panics
    ///
    /// When the pixel lies outside the image.
    pub fn pixel(&self, x: usize, y: usize) -> [u8; CHANNELS] {
        assert!(
            x < self.width && y < self.height,
            "the pixel ({x}, {y}) is outside a {}x{} image",
            self.width,
            self.height
        );
        let start = (y * self.width + x) * CHANNELS;
        self.pixels[start..start + CHANNELS]
            .try_into()
            .expect("a pixel is four channels")
    }
}
