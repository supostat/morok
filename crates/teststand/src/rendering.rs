use morok_platform::FrameImage;

use crate::element_box::ElementBox;

/// How many device pixels a case is drawn with to a CSS pixel.
pub const DEVICE_SCALE: f32 = 2.0;

/// What one side drew of a case under one color scheme: the frame on a transparent surface and
/// the boxes of the elements in document order.
#[derive(Clone, Debug, PartialEq)]
pub struct Rendering {
    pub image: FrameImage,
    pub boxes: Vec<ElementBox>,
}
