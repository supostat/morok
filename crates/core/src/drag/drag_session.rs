use crate::drag::drag_reach::DragReach;
use crate::element_id::ElementId;
use crate::transfer::payload::Payload;

/// A drag under way over a surface. While it lasts the pointer hovers nothing but the drop
/// target the drag is over: the hovered element of the tree is that target.
#[derive(Debug)]
pub(crate) struct DragSession {
    pub(crate) payload: Payload,
    /// Who carries the drag across the surface: the pointer input of the tree, or the system
    /// through the host of the tree.
    pub(crate) reach: DragReach,
    /// The drag source the pointer drags; none for a drag of the system, and once the source
    /// left the tree.
    pub(crate) source: Option<ElementId>,
}
