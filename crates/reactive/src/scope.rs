use crate::node_id::NodeId;

/// A handle to an ownership scope: disposing it disposes every signal, memo, effect and
/// scope created inside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scope {
    pub(crate) id: NodeId,
}
