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
    /// is taken as the identity and its scale as zero. The translation of a composed transform
    /// is small for anything on screen, so the shaders take it in single precision.
    pub(crate) fn of(transform: &Transform) -> Self {
        let inverse = transform.inverse().unwrap_or(Transform::IDENTITY);
        Self {
            forward_linear: [transform.a, transform.b, transform.c, transform.d],
            forward_offset: [transform.tx as f32, transform.ty as f32],
            padding_forward: [0.0, 0.0],
            inverse_linear: [inverse.a, inverse.b, inverse.c, inverse.d],
            inverse_offset: [inverse.tx as f32, inverse.ty as f32],
            scale_factor: transform.determinant().abs().sqrt(),
            padding_inverse: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use morok_core::Transform;

    use super::TransformEntry;

    #[test]
    fn a_row_far_down_a_scrolled_content_reaches_the_shaders_where_it_stands() {
        let far_down = 16_777_217.0;
        let row_in_view = Transform::translation(0.0, far_down).then(&Transform::translation(0.0, 12.0 - far_down));

        let entry = TransformEntry::of(&row_in_view);

        assert_eq!(entry.forward_offset[1].to_bits(), 12.0_f32.to_bits());
        assert_eq!(entry.inverse_offset[1].to_bits(), (-12.0_f32).to_bits());
    }
}
