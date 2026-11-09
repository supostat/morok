#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WhiteSpace {
    Normal,
    Nowrap,
    Pre,
    PreWrap,
}

impl WhiteSpace {
    /// Whether runs of white space, line breaks among them, shrink to one space.
    pub const fn collapses(self) -> bool {
        matches!(self, Self::Normal | Self::Nowrap)
    }

    /// Whether a line that does not fit breaks where the text allows it.
    pub const fn wraps(self) -> bool {
        matches!(self, Self::Normal | Self::PreWrap)
    }
}
