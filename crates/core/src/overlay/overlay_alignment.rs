/// Where an overlay lies along the edge of the anchor it lies on: for an overlay below or
/// above its anchor the edge runs left to right, for one beside it top to bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OverlayAlignment {
    /// The overlay starts where the anchor starts.
    Start,
    /// The middles of the two meet.
    Center,
    /// The overlay ends where the anchor ends.
    End,
}
