use crate::render::pixel_rect::PixelRect;

/// Consecutive instances drawn by one call under one scissor rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DrawBatch {
    pub(crate) scissor: PixelRect,
    pub(crate) first_instance: usize,
    pub(crate) instance_count: usize,
}
