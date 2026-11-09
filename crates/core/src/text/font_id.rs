/// A font at one size, as a [`TextShaper`](crate::TextShaper) names it: the tree carries the
/// name from the shaper to the renderer and reads nothing into it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontId(pub u32);
