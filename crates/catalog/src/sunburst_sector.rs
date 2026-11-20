use morok::RingSector;
use morok::style::{Color, Token};

/// One piece of a sunburst: the part of a ring it covers, the token of its color and how much
/// of that color shows over what lies behind.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SunburstSector {
    pub sector: RingSector,
    pub color: Token<Color>,
    pub opacity: f32,
}
