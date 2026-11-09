#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

impl Side {
    /// The order of values in a CSS shorthand of four sides.
    pub const ALL: [Self; 4] = [Self::Top, Self::Right, Self::Bottom, Self::Left];
}
