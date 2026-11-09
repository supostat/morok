use bytemuck::{Pod, Zeroable};

/// What every instance drawn into one target shares: the surface, and the window of it the
/// target shows.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct FrameUniforms {
    /// The size of the surface in CSS pixels.
    pub(crate) viewport_size: [f32; 2],
    /// Device pixels per CSS pixel.
    pub(crate) scale: f32,
    pub(crate) padding: f32,
    /// The device pixel of the surface at the top left corner of the target.
    pub(crate) origin: [f32; 2],
    /// The size of the target in device pixels.
    pub(crate) target_size: [f32; 2],
}
