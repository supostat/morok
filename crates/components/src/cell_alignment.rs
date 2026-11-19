/// The side of a column of a table its header and the texts of its cells stand at.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CellAlignment {
    #[default]
    Start,
    /// What numbers are set at, so that their digits line up.
    End,
}
