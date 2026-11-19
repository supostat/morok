use crate::element_id::ElementId;
use crate::transfer::payload::Payload;

/// A drag a tree hands to the system: the host of the tree begins it there and tells the tree
/// when it is over.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemDrag {
    /// The drag source the drag began on: the system shows its picture under the pointer.
    pub source: ElementId,
    pub payload: Payload,
}
