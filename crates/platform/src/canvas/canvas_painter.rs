use morok_core::CanvasId;

use crate::canvas::canvas_encoder::CanvasEncoder;

/// What the application draws into its canvases. The kit calls it in paint order, once per
/// canvas per frame, with the pass clipped to the canvas.
pub trait CanvasPainter {
    fn draw(&mut self, canvas: CanvasId, encoder: &mut CanvasEncoder<'_>);
}
