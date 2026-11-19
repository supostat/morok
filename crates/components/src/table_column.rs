use crate::cell_alignment::CellAlignment;

/// A column of a table: told apart from the other columns by its key, wherever the user moves
/// it and however wide the user makes it.
#[derive(Clone, Debug, PartialEq)]
pub struct TableColumn<ColumnKey> {
    pub key: ColumnKey,
    /// What the header of the column says.
    pub title: String,
    /// How wide the column is, in CSS pixels.
    pub width: f32,
    pub alignment: CellAlignment,
}
