/// How far a drag goes from its source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragReach {
    /// The drag stays in the application: the tree follows the pointer over its own surface
    /// and offers the drag to its drop targets, and the host carries it over the other
    /// surfaces of the application, whose trees take it as a drag that began outside them.
    Surface,
    /// The drag is handed to the system, which shows the picture of its source under the
    /// pointer and carries it to other windows and other applications. Over a surface it is
    /// offered to the drop targets as any drag of the system is.
    System,
}
