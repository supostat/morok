use crate::node_id::NodeId;

/// One end of a dependency between a reader and a source. Each node holds its end of the
/// dependency, and `slot` is where the other node holds the opposite end, so either end is
/// removed without a search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Edge {
    pub(crate) node: NodeId,
    pub(crate) slot: u32,
}
