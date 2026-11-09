/// Where a canvas lies on its surface: its content box in CSS pixels, and the scale of the
/// device pixels it is drawn with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// Device pixels per CSS pixel.
    pub scale: f32,
}
