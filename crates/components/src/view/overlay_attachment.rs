use morok_core::{ElementId, OverlayPlacement, Point, Rect};

use crate::view::overlay_anchor::OverlayAnchor;
use crate::view::overlay_interaction::OverlayInteraction;
use crate::view::overlay_place::OverlayPlace;
use crate::view::surface_id::SurfaceId;

/// What ties an overlay surface to the surface it is opened over.
#[derive(Debug)]
pub(crate) struct OverlayAttachment {
    pub(crate) parent: SurfaceId,
    /// The element of the parent the overlay belongs to; the overlay goes with it.
    pub(crate) owner: ElementId,
    pub(crate) anchor: OverlayAnchor,
    pub(crate) placement: OverlayPlacement,
    pub(crate) interaction: OverlayInteraction,
    /// The box around the panel that leaves room for the shadow of the panel.
    pub(crate) frame: ElementId,
    /// What the overlay shows: the element its content built.
    pub(crate) panel: ElementId,
    /// The rectangle of the parent the overlay is placed against, as the last frame of the
    /// parent left the anchor; none until such a frame ran.
    pub(crate) anchor_box: Option<Rect>,
    /// Where the host placed the overlay, as its content follows it.
    pub(crate) place: OverlayPlace,
    /// Where the host put the top left corner of the overlay surface, in CSS pixels of the
    /// parent; none until the host placed the overlay.
    pub(crate) origin: Option<Point>,
    /// Whether the pointer is over the room around the panel, so that what it does goes to
    /// the surface under the overlay.
    pub(crate) passes_pointer_on: bool,
}
