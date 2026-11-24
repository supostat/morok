use accesskit::{Node, SortDirection};

/// Where the node of an element stands among the items of a list, a tree, a strip of tabs or a
/// table, as an assistive technology reads it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AccessibleItem {
    /// The depth of an item of a tree, one for its top level; none outside a tree.
    pub level: Option<usize>,
    /// Whether an item that has items under it, or a box that opens a list, shows them; none
    /// for an element that has nothing to open.
    pub expanded: Option<bool>,
    /// The place of the item among its items, one for the first, and how many there are: an
    /// item may stand for one of more items than the tree holds elements for.
    pub position_in_set: Option<usize>,
    pub size_of_set: Option<usize>,
    /// How many rows and columns a table has: it may hold elements for fewer.
    pub row_count: Option<usize>,
    pub column_count: Option<usize>,
    /// The row of a row or a cell of a table and the column of a cell or a column header, zero
    /// for the first.
    pub row_index: Option<usize>,
    pub column_index: Option<usize>,
    /// The order of the rows by the column of a column header; none for a header the rows are
    /// not sorted by.
    pub sort_direction: Option<SortDirection>,
}

impl AccessibleItem {
    /// Writes into `node` every place of the item that is known.
    pub(crate) fn describe(&self, node: &mut Node) {
        if let Some(level) = self.level {
            node.set_level(level);
        }
        if let Some(expanded) = self.expanded {
            node.set_expanded(expanded);
        }
        if let Some(position) = self.position_in_set {
            node.set_position_in_set(position);
        }
        if let Some(size) = self.size_of_set {
            node.set_size_of_set(size);
        }
        if let Some(count) = self.row_count {
            node.set_row_count(count);
        }
        if let Some(count) = self.column_count {
            node.set_column_count(count);
        }
        if let Some(index) = self.row_index {
            node.set_row_index(index);
        }
        if let Some(index) = self.column_index {
            node.set_column_index(index);
        }
        if let Some(direction) = self.sort_direction {
            node.set_sort_direction(direction);
        }
    }
}
