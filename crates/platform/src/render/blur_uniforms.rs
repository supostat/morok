use bytemuck::{Pod, Zeroable};

/// One pass of a separable Gaussian blur as the shader reads it; a deviation of zero copies.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct BlurUniforms {
    /// In device pixels.
    pub(crate) deviation: f32,
    /// How many texels the kernel reaches to each side.
    pub(crate) radius: u32,
    /// `(1, 0)` or `(0, 1)`.
    pub(crate) direction: [f32; 2],
    /// The texel of the source under the top left pixel of the target.
    pub(crate) source_offset: [f32; 2],
    /// The texels of the source that hold a picture.
    pub(crate) source_size: [f32; 2],
}
