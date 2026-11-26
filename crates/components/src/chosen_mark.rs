/// Whether the list of a dropdown marks the row of the option its control holds. The control
/// says it once, when it builds the list: a column that came and went with the mark would move
/// the names of the rows under the user.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChosenMark {
    /// The row carries a check mark, and every row keeps a column for it.
    CheckMark,
    /// No row carries a mark, and no row keeps a column for one.
    NoMark,
}
