use morok_core::{OverlayInput, OverlayPlacement, Rect, Size};

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
    /// What the overlay takes of the pointer and the keyboard.
    pub input: OverlayInput,
}
