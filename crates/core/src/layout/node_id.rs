use taffy::NodeId;

use crate::element_id::ElementId;

const GENERATION_SHIFT: u32 = 32;

impl From<ElementId> for NodeId {
    fn from(id: ElementId) -> Self {
        Self::new(u64::from(id.generation()) << GENERATION_SHIFT | u64::from(id.index()))
    }
}

impl From<NodeId> for ElementId {
    fn from(node: NodeId) -> Self {
        let bits = u64::from(node);
        Self::new(bits as u32, (bits >> GENERATION_SHIFT) as u32)
    }
}

#[cfg(test)]
mod tests {
    use taffy::NodeId;

    use crate::element_id::ElementId;

    #[test]
    fn an_element_id_survives_the_round_trip_through_a_node_id() {
        let id = ElementId::new(u32::MAX - 1, 3);

        assert_eq!(ElementId::from(NodeId::from(id)), id);
    }
}
