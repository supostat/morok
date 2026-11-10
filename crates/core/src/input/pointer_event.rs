use crate::geometry::point::Point;
use crate::input::modifiers::Modifiers;
use crate::input::pointer_event_kind::PointerEventKind;

/// What the pointer did to an element.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerEvent {
    pub kind: PointerEventKind,
    /// Where on the surface, in CSS pixels.
    pub position: Point,
    pub modifiers: Modifiers,
}
