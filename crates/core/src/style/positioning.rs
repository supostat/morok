use morok_style::Position;

/// The computed `position`: an element is static until a declaration positions it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Positioning {
    Static,
    Relative,
    Absolute,
    /// Laid out in flow and shifted at paint time to stay inside the scrollport of the nearest
    /// `Scroll` ancestor; outside a `Scroll` it stays where it was laid out.
    Sticky,
}

impl Positioning {
    /// Whether the element is the containing block of its absolutely positioned descendants.
    pub const fn is_positioned(self) -> bool {
        !matches!(self, Self::Static)
    }
}

impl From<Position> for Positioning {
    fn from(position: Position) -> Self {
        match position {
            Position::Relative => Self::Relative,
            Position::Absolute => Self::Absolute,
            Position::Sticky => Self::Sticky,
        }
    }
}
