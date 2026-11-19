use crate::element_id::ElementId;
use crate::geometry::point::Point;

/// A press of the primary button that becomes a drag once the pointer travels far enough.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DragCandidate {
    /// The drag source the press fell on or inside.
    pub(crate) source: ElementId,
    pub(crate) pressed_at: Point,
}

impl DragCandidate {
    /// How far the pointer travels with the button down before the press is a drag, in CSS
    /// pixels.
    const SLOP: f32 = 4.0;

    pub(crate) fn has_travelled_to(&self, position: Point) -> bool {
        (position.x - self.pressed_at.x).hypot(position.y - self.pressed_at.y) >= Self::SLOP
    }
}
