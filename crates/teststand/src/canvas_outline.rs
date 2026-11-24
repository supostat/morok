use morok_core::Rect;
use morok_platform::RingSector;

/// The outline of a shape on a canvas, in the CSS pixels of the content box of the canvas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CanvasOutline {
    /// The part of a ring between two angles.
    Ring(RingSector),
    Rectangle(Rect),
}
