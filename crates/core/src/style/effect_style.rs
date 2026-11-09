use morok_style::BlendMode;

use crate::style::declaration_source::DeclarationSource;
use crate::style::resolved_length::ResolvedLength;

/// The part of a computed style that composites the element and its content as a group: a
/// transform of the whole subtree and the effects that need the subtree drawn apart first.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectStyle {
    /// From `0.0`, invisible, to `1.0`, opaque.
    pub opacity: f32,
    /// The transform list; none when it is empty.
    pub(crate) transform: Option<DeclarationSource>,
    /// From the left edge of the border box.
    pub transform_origin_x: ResolvedLength,
    /// From the top edge of the border box.
    pub transform_origin_y: ResolvedLength,
    /// The filter list; none when it is empty.
    pub(crate) filter: Option<DeclarationSource>,
    /// The backdrop filter list; none when it is empty.
    pub(crate) backdrop_filter: Option<DeclarationSource>,
    pub mix_blend_mode: BlendMode,
}

impl EffectStyle {
    pub(crate) const INITIAL: Self = Self {
        opacity: 1.0,
        transform: None,
        transform_origin_x: ResolvedLength {
            pixels: 0.0,
            fraction: Some(0.5),
        },
        transform_origin_y: ResolvedLength {
            pixels: 0.0,
            fraction: Some(0.5),
        },
        filter: None,
        backdrop_filter: None,
        mix_blend_mode: BlendMode::Normal,
    };

    /// Whether the element and its content are drawn apart and composited as one picture:
    /// group opacity, a filter, a backdrop filter or a blend mode asks for it.
    pub fn needs_offscreen_pass(&self) -> bool {
        self.opacity < 1.0
            || self.filter.is_some()
            || self.backdrop_filter.is_some()
            || self.mix_blend_mode != BlendMode::Normal
    }

    /// Whether the element paints as a stacking context of its own: above the flow, with its
    /// positioned descendants inside it.
    pub fn creates_stacking_context(&self) -> bool {
        self.transform.is_some() || self.needs_offscreen_pass()
    }
}
