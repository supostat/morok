use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::input::modifiers::Modifiers;
use crate::input::pointer_event_kind::PointerEventKind;

/// What the pointer did to an element.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerEvent {
    pub kind: PointerEventKind,
    /// Where on the surface, in CSS pixels.
    pub position: Point,
    pub modifiers: Modifiers,
    /// The box of the element that hears of the event, as the surface showed it in the last
    /// frame: around the border box moved by the transforms of the element and of its
    /// ancestors, as a browser's bounding client rect is. Where the pointer is against the
    /// element is `position` against this box.
    pub element_box: Rect,
}
