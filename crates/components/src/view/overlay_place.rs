use morok_core::OverlaySide;
use morok_reactive::Signal;

/// Where the host placed an overlay, as signals the content of the overlay follows.
#[derive(Clone, Copy, Debug)]
pub struct OverlayPlace {
    /// The side of the anchor the overlay lies on: it changes when the overlay flips.
    pub side: Signal<OverlaySide>,
    /// How far the edges of the screen moved the overlay along that side from where its
    /// placement put it, in CSS pixels: to the right or downwards when positive.
    pub edge_shift: Signal<f32>,
}
