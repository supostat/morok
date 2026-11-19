#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragEventKind {
    /// The drag came over the target.
    Entered,
    /// The drag moved over the target; told after it came over it as well.
    Moved,
    /// The drag is no longer over the target: it moved on, or it ended somewhere else.
    Left,
    /// The drag ended over the target: what it carries is the target's now.
    Dropped,
}
