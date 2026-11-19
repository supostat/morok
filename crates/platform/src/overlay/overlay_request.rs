use morok_core::{OverlayPlacement, Rect, Size};

use crate::overlay::overlay_id::OverlayId;

/// An overlay the content of a surface wants shown over it: a surface of its own, in a window
/// without a frame that may reach past the window it belongs to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayRequest {
    pub id: OverlayId,
    /// The rectangle of the surface the overlay is placed against, in CSS pixels of the surface.
    pub anchor: Rect,
    pub placement: OverlayPlacement,
    /// The size of the surface of the overlay, in CSS pixels.
    pub size: Size,
    /// The box on the surface of the overlay that is placed against the anchor and kept on
    /// the screen; what lies around it is room for its shadow.
    pub panel: Rect,
    /// Whether the overlay takes the pointer and the keyboard; one that does not lets both
    /// through to what lies under it.
    pub takes_input: bool,
}
