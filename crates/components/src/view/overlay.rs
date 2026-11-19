use morok_core::OverlayPlacement;

use crate::view::overlay_anchor::OverlayAnchor;
use crate::view::overlay_interaction::OverlayInteraction;

/// An overlay as a component asks for it: a surface of its own, shown by the host in a
/// window without a frame that may reach past the window it is opened over.
#[derive(Debug)]
pub struct Overlay {
    pub anchor: OverlayAnchor,
    pub placement: OverlayPlacement,
    pub interaction: OverlayInteraction,
}
