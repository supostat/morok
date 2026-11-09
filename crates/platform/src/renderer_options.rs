/// How much memory the atlases and the image textures of a renderer may take.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RendererOptions {
    /// The side of a square atlas page, in texels.
    pub atlas_page_side: u32,
    /// How many pages an atlas holds before it drops its least recently used one.
    pub atlas_page_budget: usize,
    /// How many bytes the textures of large images may take before the least recently drawn
    /// ones are dropped.
    pub image_byte_budget: usize,
}

const MEBIBYTE: usize = 1024 * 1024;

impl Default for RendererOptions {
    fn default() -> Self {
        Self {
            atlas_page_side: 1024,
            atlas_page_budget: 8,
            image_byte_budget: 64 * MEBIBYTE,
        }
    }
}
