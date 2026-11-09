use bytemuck::{Pod, Zeroable};
use morok_core::Transform;

/// A transform of the surface as the shaders read it: the matrix, its inverse and the factor
/// it scales lengths by, so that an edge stays a device pixel wide.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct TransformEntry {
    /// `a, b, c, d` of the matrix.
    pub(crate) forward_linear: [f32; 4],
    /// `tx, ty` of the matrix.
    pub(crate) forward_offset: [f32; 2],
    pub(crate) padding_forward: [f32; 2],
    pub(crate) inverse_linear: [f32; 4],
    pub(crate) inverse_offset: [f32; 2],
    pub(crate) scale_factor: f32,
    pub(crate) padding_inverse: f32,
}

impl TransformEntry {
    pub(crate) const IDENTITY: Self = Self {
        forward_linear: [1.0, 0.0, 0.0, 1.0],
        forward_offset: [0.0, 0.0],
        padding_forward: [0.0, 0.0],
        inverse_linear: [1.0, 0.0, 0.0, 1.0],
        inverse_offset: [0.0, 0.0],
        scale_factor: 1.0,
        padding_inverse: 0.0,
    };

    /// The entry of a transform; one that collapses the plane draws nothing, so its inverse
    /// is taken as the identity and its scale as zero.
    pub(crate) fn of(transform: &Transform) -> Self {
        let inverse = transform.inverse().unwrap_or(Transform::IDENTITY);
        Self {
            forward_linear: [transform.a, transform.b, transform.c, transform.d],
            forward_offset: [transform.tx, transform.ty],
            padding_forward: [0.0, 0.0],
            inverse_linear: [inverse.a, inverse.b, inverse.c, inverse.d],
            inverse_offset: [inverse.tx, inverse.ty],
            scale_factor: transform.determinant().abs().sqrt(),
            padding_inverse: 0.0,
        }
    }
}
