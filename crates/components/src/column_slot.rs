use morok_reactive::Memo;
use morok_style::Declaration;

/// What a cell slot of a table shows of the column it is bound to now: the header and every
/// row hold one slot of each number, all bound to the same column.
pub(crate) struct ColumnSlot<ColumnKey> {
    /// The place of the column among the columns, zero for the leftmost.
    pub(crate) column: Memo<usize>,
    pub(crate) key: Memo<ColumnKey>,
    /// Where the left edge of the column lies across the content of the table.
    pub(crate) left: Memo<f32>,
    pub(crate) width: Memo<Declaration>,
    pub(crate) is_end_aligned: Memo<bool>,
}

impl<ColumnKey> Clone for ColumnSlot<ColumnKey> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<ColumnKey> Copy for ColumnSlot<ColumnKey> {}
