use crate::drag::drag_reach::DragReach;
use crate::transfer::payload::Payload;

/// What a drag source answers the pointer that begins to drag it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DragStart {
    /// What the drag carries.
    pub payload: Payload,
    pub reach: DragReach,
}
