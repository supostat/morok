/// What becomes of content that does not fit its box along one axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Overflow {
    Visible,
    Hidden,
    /// Clipped, and scrolled when the content does not fit.
    Auto,
    /// Clipped, and scrolled whether the content fits or not.
    Scroll,
}
