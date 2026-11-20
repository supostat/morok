use morok_platform::RingSector;
use morok_style::{Color, Token};

/// A shape a case fills a canvas with, in the color of a token: the kit draws it through the
/// fills of its canvas encoder, Chrome as a path of an SVG in the place of the canvas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasShape {
    pub sector: RingSector,
    pub color: Token<Color>,
    /// How much of the color shows over what lies behind.
    pub opacity: f32,
}
