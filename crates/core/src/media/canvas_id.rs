/// A canvas as the application names it: the renderer hands the name back when the canvas is
/// to be drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CanvasId(pub u32);
