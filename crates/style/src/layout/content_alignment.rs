/// A value of `justify-content` or `align-content`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ContentAlignment {
    Normal,
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}
