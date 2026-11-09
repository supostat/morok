use crate::render::clip_shape::ClipShape;
use crate::render::pixel_rect::PixelRect;

/// A group drawn apart and composited onto its target: what its picture covers and how it is
/// laid over the target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GroupPass {
    /// The device pixels of the surface the picture covers, the reach of its blur included.
    /// Known once the group has ended.
    pub(crate) pixels: PixelRect,
    pub(crate) opacity: f32,
    pub(crate) blend_mode: u32,
    /// The color filters of the picture, a run of the filter buffer of the frame.
    pub(crate) filters_start: u32,
    pub(crate) filters_count: u32,
    /// The deviation of the blur of the picture in device pixels; none for no blur.
    pub(crate) blur: Option<f32>,
    /// What lies under the group is filtered inside the frame of the element before the group
    /// is drawn.
    pub(crate) backdrop: Option<BackdropPass>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BackdropPass {
    /// The device pixels of the surface under the frame, the reach of the blur included.
    pub(crate) pixels: PixelRect,
    pub(crate) filters_start: u32,
    pub(crate) filters_count: u32,
    pub(crate) blur: Option<f32>,
    /// The border box of the element: the filtered backdrop shows inside it only.
    pub(crate) clip: ClipShape,
}
