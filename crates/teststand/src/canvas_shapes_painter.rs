use morok_core::CanvasId;
use morok_platform::{CanvasEncoder, CanvasPainter};

use crate::canvas_shape::CanvasShape;

/// Fills the canvases of a scene with the shapes the case gave each, in the colors the tokens
/// have under the theme of the surface.
#[derive(Debug)]
pub(crate) struct CanvasShapesPainter {
    canvases: Vec<(CanvasId, Vec<CanvasShape>)>,
}

impl CanvasShapesPainter {
    pub(crate) fn new(canvases: Vec<(CanvasId, Vec<CanvasShape>)>) -> Self {
        Self { canvases }
    }
}

impl CanvasPainter for CanvasShapesPainter {
    fn draw(&mut self, canvas: CanvasId, encoder: &mut CanvasEncoder<'_>) {
        let Some((_, shapes)) = self.canvases.iter().find(|(drawn, _)| *drawn == canvas) else {
            return;
        };
        for shape in shapes {
            let mut color = *encoder.theme().resolve(shape.color);
            color.alpha *= shape.opacity;
            encoder.fill_ring_sector(shape.sector, color);
        }
    }
}
