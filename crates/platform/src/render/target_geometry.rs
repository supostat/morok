use morok_core::Size;

use crate::render::pixel_rect::PixelRect;

/// The size of a render target in the two units a frame works in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TargetGeometry {
    /// The size of the surface in CSS pixels.
    pub(crate) viewport: Size,
    /// Device pixels per CSS pixel.
    pub(crate) scale: f32,
    pub(crate) pixel_width: usize,
    pub(crate) pixel_height: usize,
}

impl TargetGeometry {
    pub(crate) fn new(viewport: Size, scale: f32) -> Self {
        Self {
            viewport,
            scale,
            pixel_width: (viewport.width * scale).round() as usize,
            pixel_height: (viewport.height * scale).round() as usize,
        }
    }

    /// Whether the target has no pixel to draw into.
    pub(crate) fn is_empty(&self) -> bool {
        self.pixel_width == 0 || self.pixel_height == 0
    }

    pub(crate) fn pixels(&self) -> PixelRect {
        PixelRect {
            x: 0,
            y: 0,
            width: self.pixel_width,
            height: self.pixel_height,
        }
    }
}
