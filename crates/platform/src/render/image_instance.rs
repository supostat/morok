use bytemuck::{Pod, Zeroable};

/// One instanced quad of the image pipeline: a part of a picture stretched over a quad.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct ImageInstance {
    /// The quad in CSS pixels on the surface, origin and size.
    pub(crate) bounds: [f32; 4],
    /// The texels of the picture under the quad, origin and size.
    pub(crate) texels: [f32; 4],
    /// The rounded clips the instance is masked by, in the clip buffer of the frame.
    pub(crate) clips_start: u32,
    pub(crate) clips_count: u32,
    /// The transform of the instance, in the transform buffer of the frame.
    pub(crate) transform: u32,
    pub(crate) padding: u32,
}
