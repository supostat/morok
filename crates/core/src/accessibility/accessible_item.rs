/// Where the node of an element stands among the items of a list, a tree or a strip of tabs,
/// as an assistive technology reads it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AccessibleItem {
    /// The depth of an item of a tree, one for its top level; none outside a tree.
    pub level: Option<usize>,
    /// Whether an item that has items under it shows them; none for an item that has none.
    pub expanded: Option<bool>,
    /// The place of the item among its items, one for the first, and how many there are: an
    /// item may stand for one of more items than the tree holds elements for.
    pub position_in_set: Option<usize>,
    pub size_of_set: Option<usize>,
}
