use std::slice;

use taffy::NodeId;

use crate::element_id::ElementId;

/// The children of an element as the node IDs Taffy walks.
#[derive(Clone, Debug)]
pub(crate) struct ChildNodes<'a>(pub(crate) slice::Iter<'a, ElementId>);

impl Iterator for ChildNodes<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        self.0.next().copied().map(NodeId::from)
    }
}
