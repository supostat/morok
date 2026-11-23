/// How far something is moved, or the content of a `Scroll` element is scrolled, in CSS pixels;
/// `y` grows downwards. Double precision: far down a long content a single-precision length has
/// steps of more than a pixel.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Offset {
    pub x: f64,
    pub y: f64,
}
