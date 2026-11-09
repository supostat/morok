use crate::geometry::rect::Rect;
use crate::paint::item_range::ItemRange;

/// What the display list holds of one element after the last paint.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PaintedElement {
    /// Whether the last display list held the element at all.
    pub(crate) is_painted: bool,
    /// Everything the element drew, shadows and outline included.
    pub(crate) visual_bounds: Rect,
    pub(crate) box_items: ItemRange,
    pub(crate) outline_items: ItemRange,
    /// For an element that paints as a group: everything the group drew on the surface, the
    /// reach of its filters included.
    pub(crate) group_bounds: Rect,
}
