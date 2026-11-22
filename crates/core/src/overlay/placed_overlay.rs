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
    /// How far the edges of the bounds moved the overlay along the edge of its anchor to keep
    /// it inside them, from where its placement put it: to the right or downwards when
    /// positive, nothing when it fit there.
    pub edge_shift: f32,
}
