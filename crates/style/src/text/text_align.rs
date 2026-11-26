/// Where the lines of a text sit across their box.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextAlign {
    Start,
    End,
    Left,
    Right,
    Center,
}
