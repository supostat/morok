/// What the style stage of a frame leaves for the stages after it.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct StyleOutcome {
    pub(crate) resolved_elements: usize,
    /// Whether an element entered, left or moved within the paint order.
    pub(crate) reorders_paint: bool,
    /// Whether an element must rebuild its display items.
    pub(crate) repaints: bool,
}
