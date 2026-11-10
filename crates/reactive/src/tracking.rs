use crate::node_id::NodeId;

/// What the code that runs right now is accounted to.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Tracking {
    /// The memo or effect that comes to depend on what is read.
    pub(crate) observer: Option<NodeId>,
    /// The node that owns what is created.
    pub(crate) owner: Option<NodeId>,
    /// The number of the run of `observer`; no run has the number zero.
    pub(crate) run: u64,
}
