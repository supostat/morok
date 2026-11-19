use morok_core::{ElementId, Point};

/// What an overlay is placed against, on the surface it is opened over.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OverlayAnchor {
    /// The box of an element, wherever the frames of the surface move it. A press on the
    /// element is not a press outside the overlay: the element is what opens and closes it.
    Element(ElementId),
    /// A point of the surface, as the place of the pointer is for a context menu.
    Point(Point),
}
