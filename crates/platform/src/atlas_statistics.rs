use crate::atlas_usage::AtlasUsage;

/// The state of the two glyph atlases of a renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AtlasStatistics {
    /// The atlas of coverage masks: glyphs of ordinary fonts.
    pub mask: AtlasUsage,
    /// The atlas of color glyphs.
    pub color: AtlasUsage,
}
