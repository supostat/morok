use morok_core::accesskit::SortDirection;

/// The order of the rows of a table by the values of a column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    /// The order a click on the header of a column the rows are sorted by in this order asks for.
    pub(crate) fn reversed(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

impl From<SortOrder> for SortDirection {
    fn from(order: SortOrder) -> Self {
        match order {
            SortOrder::Ascending => Self::Ascending,
            SortOrder::Descending => Self::Descending,
        }
    }
}
