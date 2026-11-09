use morok_core::CanvasId;

use crate::render::draw_batch::DrawBatch;
use crate::render::pixel_rect::PixelRect;

/// One step of drawing a frame, in order. The batches between the begin and the end of a group
/// are drawn into a picture of their own, which the end lays over the target the group began on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FrameCommand {
    Draw(DrawBatch),
    /// Begins the group at this position of the groups of the frame.
    BeginGroup(usize),
    EndGroup(usize),
    /// Hands the pass to the application for the canvas, clipped to `scissor`.
    Canvas {
        canvas: CanvasId,
        /// The content box of the element in device pixels of the surface.
        pixels: PixelRect,
        scissor: PixelRect,
        /// The transform of the canvas, in the transform buffer of the frame.
        transform: u32,
    },
}
