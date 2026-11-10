use accesskit::NodeId;

use crate::element_id::ElementId;

const INDEX_BITS: u32 = 32;

/// The node of an element is named by the handle of the element, so a slot reused by another
/// element names another node.
impl From<ElementId> for NodeId {
    fn from(element: ElementId) -> Self {
        Self(u64::from(element.generation()) << INDEX_BITS | u64::from(element.index()))
    }
}

/// The handle a node was named by. It names an element only for as long as the element is in
/// its tree.
pub(crate) fn element_of(node: NodeId) -> ElementId {
    let index = (node.0 & u64::from(u32::MAX)) as u32;
    let generation = (node.0 >> INDEX_BITS) as u32;
    ElementId::new(index, generation)
}

#[cfg(test)]
mod tests {
    use accesskit::NodeId;

    use super::element_of;
    use crate::element_id::ElementId;

    #[test]
    fn a_node_is_named_by_the_index_and_the_generation_of_its_element() {
        let element = ElementId::new(7, 3);

        let node = NodeId::from(element);

        assert_eq!(node, NodeId((3 << 32) | 7));
        assert_eq!(element_of(node), element);
    }
}
