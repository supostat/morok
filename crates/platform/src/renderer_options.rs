/// How much memory the glyph atlases of a renderer may take.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RendererOptions {
    /// The side of a square atlas page, in texels.
    pub atlas_page_side: u32,
    /// How many pages an atlas holds before it drops its least recently used one.
    pub atlas_page_budget: usize,
}

impl Default for RendererOptions {
    fn default() -> Self {
        Self {
            atlas_page_side: 1024,
            atlas_page_budget: 8,
        }
    }
}
