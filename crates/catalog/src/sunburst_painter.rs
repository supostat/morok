use morok::core::CanvasId;
use morok::{CanvasEncoder, CanvasPainter};

use crate::sunburst_sector::SunburstSector;

/// Draws a sunburst into one canvas in the colors the tokens of the surface give it now: a
/// change of the theme or of the token set repaints it in the colors of the new values.
#[derive(Clone, Debug)]
pub struct SunburstPainter {
    canvas: CanvasId,
    sectors: Vec<SunburstSector>,
}

impl SunburstPainter {
    /// A painter of `sectors` into the canvas named `canvas`; it leaves other canvases alone.
    pub fn new(canvas: CanvasId, sectors: Vec<SunburstSector>) -> Self {
        Self { canvas, sectors }
    }
}

impl CanvasPainter for SunburstPainter {
    fn draw(&mut self, canvas: CanvasId, encoder: &mut CanvasEncoder<'_>) {
        if canvas != self.canvas {
            return;
        }
        for piece in &self.sectors {
            let mut color = *encoder.theme().resolve(piece.color);
            color.alpha *= piece.opacity;
            encoder.fill_ring_sector(piece.sector, color);
        }
    }
}
