use crate::color::Color;
use crate::pixels::Pixels;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowLayer {
    pub offset_x: Pixels,
    pub offset_y: Pixels,
    pub blur: Pixels,
    pub spread: Pixels,
    pub color: Color,
    pub inset: bool,
}
