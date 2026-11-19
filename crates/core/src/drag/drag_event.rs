use crate::drag::drag_event_kind::DragEventKind;
use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::input::modifiers::Modifiers;
use crate::transfer::payload::Payload;

/// What a drag did to a drop target that takes a type the drag carries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragEvent<'a> {
    pub kind: DragEventKind,
    /// Where the drag is on the surface, in CSS pixels.
    pub position: Point,
    pub modifiers: Modifiers,
    /// What the drag carries.
    pub payload: &'a Payload,
    /// The box of the target as the surface showed it in the last frame, as
    /// [`PointerEvent::element_box`](crate::PointerEvent::element_box) is.
    pub element_box: Rect,
}
