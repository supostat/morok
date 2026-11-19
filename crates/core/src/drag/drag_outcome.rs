/// How a drag ended, as its source hears of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragOutcome {
    /// A drop target took what the drag carried.
    Dropped,
    /// The drag ended over nothing that takes it.
    Cancelled,
}
