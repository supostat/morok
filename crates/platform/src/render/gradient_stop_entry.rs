use bytemuck::{Pod, Zeroable};

/// One stop of a gradient as the shader reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct GradientStopEntry {
    pub(crate) color: [f32; 4],
    pub(crate) offset: f32,
    pub(crate) padding: [f32; 3],
}
