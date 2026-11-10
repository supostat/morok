use morok_core::ElementId;

/// A handle to a fragment of the children of an element: a run of children that come and go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FragmentId {
    pub(crate) parent: ElementId,
    /// The place of the fragment among the fragments of its parent, in the order of the children.
    pub(crate) position: usize,
}
