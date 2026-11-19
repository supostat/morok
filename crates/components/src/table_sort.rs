use crate::sort_order::SortOrder;

/// What the rows of a table are sorted by: a column, by its key, and the order of its values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableSort<ColumnKey> {
    pub column: ColumnKey,
    pub order: SortOrder,
}
