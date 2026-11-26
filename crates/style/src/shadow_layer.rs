use crate::color::Color;
use crate::pixels::Pixels;

/// One layer of a box shadow: where it falls, how far it blurs and spreads, its color, and
/// whether it is drawn inside the box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowLayer {
    pub offset_x: Pixels,
    pub offset_y: Pixels,
    pub blur: Pixels,
    pub spread: Pixels,
    pub color: Color,
    pub inset: bool,
}
