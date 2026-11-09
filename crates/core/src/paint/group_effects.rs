use morok_style::BlendMode;

use crate::geometry::transform::Transform;
use crate::paint::box_frame::BoxFrame;
use crate::paint::item_range::ItemRange;

/// How a group — an element with its content — is placed and composited onto what lies under
/// it. The items between the group's push and pop are drawn in the coordinates of the surface
/// before the transform, and the transform moves them as one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupEffects {
    /// The transform of the group on the surface, enclosing groups included.
    pub transform: Transform,
    /// From `0.0`, invisible, to `1.0`, opaque.
    pub opacity: f32,
    /// The filters of the group's own picture, see [`DisplayList::filters`](crate::DisplayList::filters).
    pub filter: ItemRange,
    /// The filters of what lies under the group, applied inside the frame before the group draws.
    pub backdrop_filter: ItemRange,
    pub mix_blend_mode: BlendMode,
    /// The border box of the element the group belongs to, as laid out: the backdrop is
    /// filtered inside it.
    pub frame: BoxFrame,
}

impl GroupEffects {
    /// Whether the group is drawn apart and composited as one picture.
    pub fn needs_offscreen_pass(&self) -> bool {
        self.opacity < 1.0
            || self.filter.start != self.filter.end
            || self.backdrop_filter.start != self.backdrop_filter.end
            || self.mix_blend_mode != BlendMode::Normal
    }
}
