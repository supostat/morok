use morok_style::{Color, ShadowLayer};

use crate::geometry::sides::Sides;
use crate::paint::box_frame::BoxFrame;
use crate::paint::gradient_fill::GradientFill;

/// One primitive of a display list. Items are in paint order: a later item is drawn over an
/// earlier one. An item states what CSS asks to be drawn; how it is rasterized is the renderer's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisplayItem {
    /// One layer of a box shadow: outside the border box, or inside the padding box when the
    /// layer is inset.
    Shadow {
        frame: BoxFrame,
        layer: ShadowLayer,
    },
    /// The background color, filling the border box.
    Background {
        frame: BoxFrame,
        color: Color,
    },
    /// The background image: one tile covers the padding box and repeats under the borders.
    Gradient {
        frame: BoxFrame,
        gradient: GradientFill,
    },
    Border {
        frame: BoxFrame,
        colors: Sides<Color>,
    },
    /// A line of `width` around the border box, `offset` away from it.
    Outline {
        frame: BoxFrame,
        width: f32,
        offset: f32,
        color: Color,
    },
    /// Clips the items up to the matching [`DisplayItem::PopClip`] to the padding box of the frame.
    PushClip {
        frame: BoxFrame,
    },
    PopClip,
}
