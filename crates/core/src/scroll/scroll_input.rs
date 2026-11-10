use crate::geometry::point::Point;
use crate::scroll::scroll_phase::ScrollPhase;

/// What the wheel or the trackpad did over a surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollInput {
    /// Where on the surface the pointer was, in CSS pixels.
    pub position: Point,
    /// How far the content is to scroll, in CSS pixels: positive towards its end, down and to
    /// the right.
    pub delta: Point,
    pub phase: ScrollPhase,
}
