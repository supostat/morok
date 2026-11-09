/// How full one glyph atlas is and what it has done since the renderer was created.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AtlasUsage {
    pub pages: usize,
    /// The shelves in use over every page.
    pub shelves: usize,
    /// The glyphs the atlas holds now.
    pub glyphs: usize,
    /// Every glyph rasterized into the atlas, those rasterized again after an eviction among them.
    pub rasterized_glyphs: u64,
    pub evicted_pages: u64,
}
