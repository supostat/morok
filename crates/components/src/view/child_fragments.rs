use std::collections::HashMap;

use morok_core::ElementId;

use crate::view::fragment_extent::FragmentExtent;
use crate::view::fragment_id::FragmentId;

/// The fragments of the children of every element of a surface: where the children that come
/// and go lie among the children that were mounted for good.
#[derive(Debug, Default)]
pub(crate) struct ChildFragments {
    of_parents: HashMap<ElementId, Vec<FragmentExtent>>,
}

impl ChildFragments {
    /// Opens an empty fragment after the `children_now` children `parent` has.
    pub(crate) fn open(&mut self, parent: ElementId, children_now: usize) -> FragmentId {
        let fragments = self.of_parents.entry(parent).or_default();
        let children_accounted: usize = fragments
            .iter()
            .map(|fragment| fragment.fixed_children_before + fragment.length)
            .sum();
        fragments.push(FragmentExtent {
            fixed_children_before: children_now - children_accounted,
            length: 0,
        });
        FragmentId {
            parent,
            position: fragments.len() - 1,
        }
    }

    /// The place of the first child of the fragment among the children of its parent.
    pub(crate) fn start(&self, fragment: FragmentId) -> usize {
        let fragments = &self.of_parents[&fragment.parent];
        let children_of_earlier_fragments: usize = fragments[..fragment.position]
            .iter()
            .map(|earlier| earlier.fixed_children_before + earlier.length)
            .sum();
        children_of_earlier_fragments + fragments[fragment.position].fixed_children_before
    }

    pub(crate) fn set_length(&mut self, fragment: FragmentId, length: usize) {
        self.of_parents
            .get_mut(&fragment.parent)
            .expect("a fragment is resized while its parent is in the tree")[fragment.position]
            .length = length;
    }

    /// The element is removed: its fragments go with it.
    pub(crate) fn forget_parent(&mut self, parent: ElementId) {
        self.of_parents.remove(&parent);
    }
}
