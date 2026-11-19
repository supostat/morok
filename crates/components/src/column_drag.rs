/// The press of the pointer on the header of a column, which becomes a drag of the column
/// across the others once the pointer travels far enough. It is kept after the button comes up
/// for the click that follows.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ColumnDrag<ColumnKey> {
    pub(crate) key: ColumnKey,
    /// Where the pointer went down, across the surface.
    pub(crate) pressed_at: f32,
    /// Where the pointer went down, across the content of the table.
    pub(crate) pressed_in_content: f32,
    pub(crate) phase: DragPhase,
}

/// How far a press on a header has come.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DragPhase {
    /// The button is down and the pointer has not travelled.
    Pressed,
    /// The button is down and the column follows the pointer.
    Moving,
    /// The button came up where it went down; the click that follows sorts by the column.
    Released,
    /// The button came up after the column was moved; the click that follows sorts nothing.
    Dropped,
}

impl DragPhase {
    /// The phase the release of the button ends this one in.
    pub(crate) fn released(self) -> Self {
        match self {
            Self::Pressed | Self::Released => Self::Released,
            Self::Moving | Self::Dropped => Self::Dropped,
        }
    }
}
