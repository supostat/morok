use crate::render::draw_batch::DrawBatch;

/// One step of drawing a frame, in order. The batches between the begin and the end of a group
/// are drawn into a picture of their own, which the end lays over the target the group began on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FrameCommand {
    Draw(DrawBatch),
    /// Begins the group at this position of the groups of the frame.
    BeginGroup(usize),
    EndGroup(usize),
}
