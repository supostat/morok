#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

impl Corner {
    /// The order of values in the `border-radius` shorthand.
    pub const ALL: [Self; 4] = [Self::TopLeft, Self::TopRight, Self::BottomRight, Self::BottomLeft];
}
