#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RepeatCount {
    Times(u16),
    AutoFill,
    AutoFit,
}
