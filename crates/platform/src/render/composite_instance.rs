use bytemuck::{Pod, Zeroable};

use crate::render::clip_shape::ClipShape;

/// A picture laid over a target as the shaders read it: the quad it covers in device pixels of
/// the target, how it is filtered, faded and blended, and the shape it is cut to.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct CompositeInstance {
    /// Origin and size in device pixels of the target; the picture starts at its top left corner.
    pub(crate) pixels: [f32; 4],
    pub(crate) opacity: f32,
    /// A [`BlendMode`](morok_style::BlendMode) as the `BLEND_` constants of the shader number them.
    pub(crate) blend_mode: u32,
    pub(crate) filters_start: u32,
    pub(crate) filters_count: u32,
    pub(crate) clip: ClipShape,
    pub(crate) has_clip: u32,
    pub(crate) padding: [u32; 3],
}
