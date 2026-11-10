use crate::node_id::NodeId;

/// A handle to a function that runs again in the flush after a value it read last time changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Effect {
    pub(crate) id: NodeId,
}
