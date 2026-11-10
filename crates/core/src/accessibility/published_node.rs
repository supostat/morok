use accesskit::NodeId;

use crate::accessibility::node_facts::NodeFacts;

/// What the last tree update said of a node.
#[derive(Debug, Default)]
pub(crate) struct PublishedNode {
    /// The update that held the element last; no update has the number zero. An element that
    /// an update leaves out, hidden or detached, is new to the update that holds it again.
    pub(crate) update: u64,
    pub(crate) facts: NodeFacts,
    pub(crate) children: Vec<NodeId>,
}
