use bytemuck::{Pod, Zeroable};

/// What every instance of a frame shares.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct FrameUniforms {
    /// The size of the surface in CSS pixels.
    pub(crate) viewport_size: [f32; 2],
    /// Device pixels per CSS pixel.
    pub(crate) scale: f32,
    pub(crate) padding: f32,
}
