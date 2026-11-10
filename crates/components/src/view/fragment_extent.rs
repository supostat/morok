/// Where a fragment lies among the children of its parent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FragmentExtent {
    /// The children mounted for good between the fragment before this one, or the first child,
    /// and this fragment.
    pub(crate) fixed_children_before: usize,
    /// The children the fragment holds now.
    pub(crate) length: usize,
}
