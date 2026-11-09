/// What the textures of large images have done since the renderer was created.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageStatistics {
    /// The bytes the textures take now.
    pub texture_bytes: usize,
    /// Every upload, those after an eviction among them.
    pub uploads: u64,
    pub evictions: u64,
}
