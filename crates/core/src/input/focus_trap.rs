use crate::element_id::ElementId;

/// A subtree that focus cannot leave.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FocusTrap {
    pub(crate) root: ElementId,
    /// What had focus when the trap was set; it takes focus back when the trap is released.
    pub(crate) returns_to: Option<ElementId>,
}
