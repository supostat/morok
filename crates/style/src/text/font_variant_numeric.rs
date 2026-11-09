#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontVariantNumeric {
    Normal,
    /// Every digit takes the same advance, so that numbers line up in columns.
    TabularNums,
}
