use std::fmt::{Debug, Formatter, Result};

use morok_core::CanvasId;
use morok_platform::{CanvasEncoder, CanvasPainter};

/// The painters the kit draws the canvases of a scene with, one to a canvas.
#[derive(Default)]
pub(crate) struct CanvasPainters {
    painters: Vec<(CanvasId, Box<dyn CanvasPainter>)>,
}

impl CanvasPainters {
    /// Names who draws `canvas`, in the place of the painter named before.
    pub(crate) fn set(&mut self, canvas: CanvasId, painter: Box<dyn CanvasPainter>) {
        self.painters.retain(|(drawn, _)| *drawn != canvas);
        self.painters.push((canvas, painter));
    }
}

impl CanvasPainter for CanvasPainters {
    fn draw(&mut self, canvas: CanvasId, encoder: &mut CanvasEncoder<'_>) {
        if let Some((_, painter)) = self.painters.iter_mut().find(|(drawn, _)| *drawn == canvas) {
            painter.draw(canvas, encoder);
        }
    }
}

impl Debug for CanvasPainters {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        let canvases: Vec<CanvasId> = self.painters.iter().map(|(canvas, _)| *canvas).collect();
        formatter
            .debug_struct("CanvasPainters")
            .field("canvases", &canvases)
            .finish()
    }
}
