use crate::element_id::ElementId;

/// One entry of the paint order: a part of an element, or a clip around the entries of its content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaintStep {
    /// Shadows, background and border.
    Box(ElementId),
    Outline(ElementId),
    /// The entries up to the matching [`PaintStep::PopClip`] are clipped by this element.
    PushClip(ElementId),
    PopClip,
    /// The entries up to the matching [`PaintStep::PopGroup`] — the element, its content and
    /// its positioned descendants — are transformed and composited as one.
    PushGroup(ElementId),
    PopGroup,
    /// The entries up to the matching [`PaintStep::PopScroll`] — content of this `Scroll`
    /// element — are moved by its scroll.
    PushScroll(ElementId),
    PopScroll,
}
