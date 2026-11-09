use bytemuck::{Pod, Zeroable};
use morok_core::Filter;

/// A color filter of a group as the shaders read it; the values of `kind` are the `FILTER_`
/// constants of the shader.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct FilterEntry {
    pub(crate) kind: u32,
    pub(crate) amount: f32,
    pub(crate) padding: [f32; 2],
}

impl FilterEntry {
    pub(crate) fn of(filter: Filter) -> Self {
        let (kind, amount) = match filter {
            Filter::Blur(deviation) => (0, deviation),
            Filter::Brightness(amount) => (1, amount),
            Filter::Contrast(amount) => (2, amount),
            Filter::Grayscale(amount) => (3, amount),
            Filter::Saturate(amount) => (4, amount),
            Filter::Opacity(amount) => (5, amount),
        };
        Self {
            kind,
            amount,
            padding: [0.0; 2],
        }
    }
}
