use crate::paint::gradient_shape::GradientShape;
use crate::paint::item_range::ItemRange;

/// A gradient of a display item; its stops live in the display list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientFill {
    pub shape: GradientShape,
    /// The stops of the gradient, see [`DisplayList::gradient_stops`](crate::DisplayList::gradient_stops).
    pub stops: ItemRange,
}
