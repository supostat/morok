/// How many rows of a list or a table the user selects at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowSelection {
    /// One row: a click or an arrow key selects the row it lands on.
    Single,
    /// Any rows: Command-click adds or takes away a row, Shift-click and Shift with an arrow
    /// key select the rows from where the selection began, Space adds or takes away the row
    /// the keyboard is on.
    Multiple,
}
