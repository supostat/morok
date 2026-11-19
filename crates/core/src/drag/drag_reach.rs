/// How far a drag goes from its source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragReach {
    /// The drag stays on its surface: the tree follows the pointer and offers the drag to the
    /// drop targets of the surface.
    Surface,
    /// The drag is handed to the system, which shows the picture of its source under the
    /// pointer and carries it to other windows and other applications. Over a surface it is
    /// offered to the drop targets as any drag of the system is.
    System,
}
