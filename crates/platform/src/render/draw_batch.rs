use crate::render::atlas_kind::AtlasKind;
use crate::render::pixel_rect::PixelRect;

/// What the instances of a batch are and which pipeline draws them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BatchSource {
    Rects,
    /// Glyphs of one atlas page.
    Glyphs {
        atlas: AtlasKind,
        page: usize,
    },
}

/// Consecutive instances of one buffer drawn by one call under one scissor rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DrawBatch {
    pub(crate) source: BatchSource,
    pub(crate) scissor: PixelRect,
    pub(crate) first_instance: usize,
    pub(crate) instance_count: usize,
}
