/// The stages of the frame of a tree, declared in the order they run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FrameStage {
    /// The styles of the elements a change marked, and what the elements in motion show.
    Styles,
    /// The natural sizes of the pictures the host has decoded.
    Images,
    Layout,
    DisplayList,
}
