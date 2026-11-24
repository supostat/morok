use morok_style::{Color, Token};

use crate::canvas_outline::CanvasOutline;

/// A shape Chrome fills on the page in the place of a canvas, in the color of a token, as a
/// path of an SVG; the kit draws the canvas with the painter the case gives the scene.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasShape {
    pub outline: CanvasOutline,
    pub color: Token<Color>,
    /// How much of the color shows over what lies behind.
    pub opacity: f32,
}
