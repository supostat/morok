/// Where a grid item starts or ends on one axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GridLine {
    Auto,
    /// A line number counted from 1, or from the end of the grid when negative.
    Line(i16),
    Span(u16),
}
