use crate::arena::Arena;
use crate::edge::Edge;
use crate::node::Node;
use crate::node_id::NodeId;

const RETIRED_HANDLE: &str = "a handle is not used after the scope that owns it is disposed";

/// The structure of the signal graph: the nodes, the dependencies between readers and
/// sources, and the ownership tree along which nodes are disposed.
#[derive(Debug, Default)]
pub(crate) struct Graph {
    nodes: Arena,
}

impl Graph {
    pub(crate) fn insert(&mut self, mut node: Node, owner: Option<NodeId>) -> NodeId {
        let first_sibling = owner.and_then(|owner| self.node(owner).first_child);
        node.owner = owner;
        node.next_sibling = first_sibling;
        let id = self.nodes.insert(node);
        if let Some(first_sibling) = first_sibling {
            self.node_mut(first_sibling).previous_sibling = Some(id);
        }
        if let Some(owner) = owner {
            self.node_mut(owner).first_child = Some(id);
        }
        id
    }

    pub(crate) fn contains(&self, id: NodeId) -> bool {
        self.nodes.get(id).is_some()
    }

    pub(crate) fn find_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    /// # Panics
    ///
    /// When the node was disposed.
    pub(crate) fn node(&self, id: NodeId) -> &Node {
        self.nodes.get(id).expect(RETIRED_HANDLE)
    }

    /// # Panics
    ///
    /// When the node was disposed.
    pub(crate) fn node_mut(&mut self, id: NodeId) -> &mut Node {
        self.nodes.get_mut(id).expect(RETIRED_HANDLE)
    }

    /// Makes `reader` depend on `source`.
    pub(crate) fn link(&mut self, reader: NodeId, source: NodeId) {
        let source_slot = edge_slot(self.node(source).subscribers.len());
        let reader_slot = edge_slot(self.node(reader).sources.len());
        self.node_mut(source).subscribers.push(Edge {
            node: reader,
            slot: reader_slot,
        });
        self.node_mut(reader).sources.push(Edge {
            node: source,
            slot: source_slot,
        });
    }

    /// Forgets what `reader` depends on; its next run collects the dependencies again.
    pub(crate) fn unlink_sources(&mut self, reader: NodeId) {
        while let Some(edge) = self.node_mut(reader).sources.pop() {
            let subscribers = &mut self.node_mut(edge.node).subscribers;
            subscribers.swap_remove(edge.slot as usize);
            if let Some(moved) = subscribers.get(edge.slot as usize).copied() {
                self.node_mut(moved.node).sources[moved.slot as usize].slot = edge.slot;
            }
        }
    }

    /// Disposes the node with everything it owns.
    ///
    /// # Panics
    ///
    /// When the node, or a node it owns, is running.
    pub(crate) fn dispose(&mut self, id: NodeId) {
        self.detach_from_owner(id);
        self.dispose_subtree(id);
    }

    /// Disposes everything the node owns and keeps the node.
    pub(crate) fn dispose_children(&mut self, id: NodeId) {
        while let Some(child) = self.node(id).first_child {
            self.node_mut(id).first_child = self.node(child).next_sibling;
            self.dispose_subtree(child);
        }
    }

    fn dispose_subtree(&mut self, id: NodeId) {
        assert!(
            !self.node(id).is_running,
            "a memo or an effect is not disposed while it runs"
        );
        self.dispose_children(id);
        self.unlink_sources(id);
        self.unlink_subscribers(id);
        self.nodes.remove(id);
    }

    fn detach_from_owner(&mut self, id: NodeId) {
        let node = self.node(id);
        let (owner, previous_sibling, next_sibling) = (node.owner, node.previous_sibling, node.next_sibling);
        if let Some(previous_sibling) = previous_sibling {
            self.node_mut(previous_sibling).next_sibling = next_sibling;
        } else if let Some(owner) = owner {
            self.node_mut(owner).first_child = next_sibling;
        }
        if let Some(next_sibling) = next_sibling {
            self.node_mut(next_sibling).previous_sibling = previous_sibling;
        }
    }

    fn unlink_subscribers(&mut self, source: NodeId) {
        while let Some(edge) = self.node_mut(source).subscribers.pop() {
            let sources = &mut self.node_mut(edge.node).sources;
            sources.swap_remove(edge.slot as usize);
            if let Some(moved) = sources.get(edge.slot as usize).copied() {
                self.node_mut(moved.node).subscribers[moved.slot as usize].slot = edge.slot;
            }
        }
    }
}

fn edge_slot(position: usize) -> u32 {
    u32::try_from(position).expect("a node has fewer than 2^32 dependencies")
}
