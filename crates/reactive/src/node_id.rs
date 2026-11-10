/// A handle to a node of the graph.
///
/// Disposing a node retires its handle: the slot may be reused, but only under a new generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct NodeId {
    index: u32,
    generation: u32,
}

impl NodeId {
    pub(crate) const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    pub(crate) const fn index(self) -> u32 {
        self.index
    }

    pub(crate) const fn generation(self) -> u32 {
        self.generation
    }
}
