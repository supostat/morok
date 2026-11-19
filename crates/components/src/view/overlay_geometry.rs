use morok_core::{OverlayPlacement, Rect, Size};

/// Where the host of a surface shows an overlay opened over it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayGeometry {
    /// The rectangle the overlay is placed against, in CSS pixels of the surface it is
    /// opened over.
    pub anchor: Rect,
    pub placement: OverlayPlacement,
    /// The size of the overlay surface.
    pub size: Size,
    /// The box of the panel on the overlay surface: what is placed against the anchor. The
    /// rest of the surface is room for the shadow of the panel.
    pub panel: Rect,
    /// Whether the overlay takes the pointer and the keyboard.
    pub takes_input: bool,
}
