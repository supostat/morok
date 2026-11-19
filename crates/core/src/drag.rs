//! Drag and drop: the drag the pointer carries from a drag source across the drop targets of
//! its surface, the drag handed to the system, and the drag of the system that comes over a
//! surface from outside.

pub(crate) mod accepted_drop_types;
pub(crate) mod drag_candidate;
pub(crate) mod drag_event;
pub(crate) mod drag_event_kind;
pub(crate) mod drag_outcome;
pub(crate) mod drag_reach;
pub(crate) mod drag_role;
pub(crate) mod drag_router;
pub(crate) mod drag_session;
pub(crate) mod drag_start;
pub(crate) mod system_drag;
