/// The main axis of a flex container and the way its items run along it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}
