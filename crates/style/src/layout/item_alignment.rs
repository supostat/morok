/// A value of `justify-items` or `align-items`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemAlignment {
    Normal,
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}
