use crate::edge::Edge;
use crate::node_content::NodeContent;
use crate::node_id::NodeId;
use crate::node_state::NodeState;

#[derive(Debug)]
pub(crate) struct Node<World> {
    pub(crate) content: NodeContent<World>,
    pub(crate) state: NodeState,
    pub(crate) owner: Option<NodeId>,
    pub(crate) first_child: Option<NodeId>,
    pub(crate) next_sibling: Option<NodeId>,
    pub(crate) previous_sibling: Option<NodeId>,
    pub(crate) sources: Vec<Edge>,
    pub(crate) subscribers: Vec<Edge>,
    /// The run that read the node last: a run that reads a source twice depends on it once.
    pub(crate) read_in_run: u64,
    pub(crate) is_running: bool,
    pub(crate) is_queued: bool,
    /// The flush `runs_in_flush` counts for.
    pub(crate) counted_flush: u64,
    pub(crate) runs_in_flush: u32,
}

impl<World> Node<World> {
    pub(crate) fn new(content: NodeContent<World>, state: NodeState) -> Self {
        Self {
            content,
            state,
            owner: None,
            first_child: None,
            next_sibling: None,
            previous_sibling: None,
            sources: Vec::new(),
            subscribers: Vec::new(),
            read_in_run: 0,
            is_running: false,
            is_queued: false,
            counted_flush: 0,
            runs_in_flush: 0,
        }
    }
}
