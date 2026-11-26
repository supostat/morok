/// What a line that does not fit its box shows where it is cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextOverflow {
    Clip,
    Ellipsis,
}
