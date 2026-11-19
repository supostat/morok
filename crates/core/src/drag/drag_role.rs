use crate::transfer::payload_type::PayloadType;

/// What an element is to a drag: a source the pointer drags, a target drags are dropped on,
/// or both.
#[derive(Debug, Default)]
pub(crate) struct DragRole {
    /// Whether the pointer may drag the element.
    pub(crate) is_source: bool,
    /// The payload types a drag is dropped on the element in; empty unless the element is a
    /// drop target.
    pub(crate) drop_types: Box<[PayloadType]>,
}
