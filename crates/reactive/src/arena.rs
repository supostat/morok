use crate::node::Node;
use crate::node_id::NodeId;

#[derive(Debug)]
struct Slot {
    generation: u32,
    node: Option<Node>,
}

/// The storage of the nodes. A freed slot is reused under a new generation, so a retired
/// handle never reaches the node that took its place.
#[derive(Debug, Default)]
pub(crate) struct Arena {
    slots: Vec<Slot>,
    free_indices: Vec<u32>,
}

impl Arena {
    pub(crate) fn insert(&mut self, node: Node) -> NodeId {
        if let Some(index) = self.free_indices.pop() {
            let slot = &mut self.slots[index as usize];
            slot.node = Some(node);
            return NodeId::new(index, slot.generation);
        }
        let index = u32::try_from(self.slots.len()).expect("a graph holds fewer than 2^32 nodes");
        self.slots.push(Slot {
            generation: 0,
            node: Some(node),
        });
        NodeId::new(index, 0)
    }

    /// # Panics
    ///
    /// When the handle is retired.
    pub(crate) fn remove(&mut self, id: NodeId) -> Node {
        let slot = &mut self.slots[id.index() as usize];
        assert_eq!(slot.generation, id.generation(), "a node is removed once");
        slot.generation += 1;
        self.free_indices.push(id.index());
        slot.node.take().expect("a slot of the current generation holds a node")
    }

    pub(crate) fn get(&self, id: NodeId) -> Option<&Node> {
        let slot = &self.slots[id.index() as usize];
        if slot.generation == id.generation() {
            slot.node.as_ref()
        } else {
            None
        }
    }

    pub(crate) fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        let slot = &mut self.slots[id.index() as usize];
        if slot.generation == id.generation() {
            slot.node.as_mut()
        } else {
            None
        }
    }
}
