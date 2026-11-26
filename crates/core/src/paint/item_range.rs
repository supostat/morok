/// A run of consecutive entries in a buffer of a display list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemRange {
    pub start: u32,
    pub end: u32,
}

impl ItemRange {
    /// The run as a range of indices into its buffer.
    pub fn as_range(self) -> std::ops::Range<usize> {
        self.start as usize..self.end as usize
    }
}
