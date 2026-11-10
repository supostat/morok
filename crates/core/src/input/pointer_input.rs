use crate::geometry::point::Point;
use crate::input::modifiers::Modifiers;
use crate::input::pointer_input_kind::PointerInputKind;

/// What the pointer did on a surface, as the host of the surface reports it to the tree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerInput {
    pub kind: PointerInputKind,
    /// Where on the surface, in CSS pixels.
    pub position: Point,
    pub modifiers: Modifiers,
}
