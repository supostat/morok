use morok::core::CanvasId;
use morok::style::{Color, Theme, Token};
use morok::{CanvasEncoder, CanvasPainter};

use crate::chart_view::ChartView;
use crate::disk_map::DiskMap;
use crate::disk_map_chart::DiskMapChart;
use crate::volume::Volume;

/// Draws the charts of the disk map, every volume in every view, each into the canvas
/// [`DiskMap::canvas`] names for it, in the colors the tokens of the surface give it now: a
/// change of the theme or of the token set repaints them in the colors of the new values.
#[derive(Clone, Debug)]
pub struct DiskMapPainter {
    charts: Vec<(CanvasId, DiskMapChart)>,
}

impl DiskMapPainter {
    pub(crate) fn new() -> Self {
        let mut charts = Vec::with_capacity(Volume::ALL.len() * ChartView::ALL.len());
        for volume in Volume::ALL {
            charts.push((
                DiskMap::canvas(volume, ChartView::Sunburst),
                DiskMapChart::Sunburst(volume.sunburst().sectors()),
            ));
            charts.push((
                DiskMap::canvas(volume, ChartView::Treemap),
                DiskMapChart::Treemap(volume.treemap().tiles()),
            ));
        }
        Self { charts }
    }
}

impl CanvasPainter for DiskMapPainter {
    fn draw(&mut self, canvas: CanvasId, encoder: &mut CanvasEncoder<'_>) {
        let Some((_, chart)) = self.charts.iter().find(|(drawn, _)| *drawn == canvas) else {
            return;
        };
        match chart {
            DiskMapChart::Sunburst(sectors) => {
                for piece in sectors {
                    let color = shown_color(encoder.theme(), piece.color, piece.opacity);
                    encoder.fill_ring_sector(piece.sector, color);
                }
            }
            DiskMapChart::Treemap(tiles) => {
                for tile in tiles {
                    let color = shown_color(encoder.theme(), tile.color, tile.opacity);
                    encoder.fill(tile.rect, color);
                }
            }
        }
    }
}

/// The value of the token under the theme, showing as much of it as `opacity` says.
fn shown_color(theme: &Theme, color: Token<Color>, opacity: f32) -> Color {
    let mut shown = *theme.resolve(color);
    shown.alpha *= opacity;
    shown
}
