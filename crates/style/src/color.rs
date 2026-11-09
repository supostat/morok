/// A non-premultiplied sRGB color; `alpha` runs from 0.0 (transparent) to 1.0 (opaque).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: f32,
}
