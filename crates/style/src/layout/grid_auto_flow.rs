/// Where a grid puts the items that name no place of their own: row by row or column by column,
/// and whether it goes back to fill the gaps it left.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GridAutoFlow {
    Row,
    Column,
    RowDense,
    ColumnDense,
}
