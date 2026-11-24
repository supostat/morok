use morok::core::Rect;
use morok::style::{Color, Token};

/// One rectangle of a treemap: where it lies in the square of the chart, the token of its
/// color and how much of that color shows over what lies behind.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreemapTile {
    pub rect: Rect,
    pub color: Token<Color>,
    pub opacity: f32,
}
