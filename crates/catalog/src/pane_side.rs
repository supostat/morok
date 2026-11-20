/// One of the two panes of the file manager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaneSide {
    Left,
    Right,
}

impl PaneSide {
    pub(crate) fn other(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}
