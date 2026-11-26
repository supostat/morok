/// Whether the items of a flex container stay on one line or wrap onto further lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}
