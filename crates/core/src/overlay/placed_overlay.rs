use crate::geometry::rect::Rect;
use crate::overlay::overlay_side::OverlaySide;

/// Where an overlay came to lie.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlacedOverlay {
    /// The rectangle of the overlay, in the coordinates the anchor and the bounds were given in.
    pub frame: Rect,
    /// The side of the anchor it lies on: the side asked for, or the opposite one when the
    /// overlay flipped.
    pub side: OverlaySide,
}
