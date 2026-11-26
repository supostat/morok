/// How many times a `repeat()` of grid tracks lays its tracks out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RepeatCount {
    Times(u16),
    AutoFill,
    AutoFit,
}
