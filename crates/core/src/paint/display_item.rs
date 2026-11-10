use morok_style::{Color, ShadowLayer};

use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::geometry::sides::Sides;
use crate::media::canvas_id::CanvasId;
use crate::media::icon_id::IconId;
use crate::media::image_id::ImageId;
use crate::paint::box_frame::BoxFrame;
use crate::paint::gradient_fill::GradientFill;
use crate::paint::group_effects::GroupEffects;
use crate::paint::item_range::ItemRange;
use crate::text::font_id::FontId;

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
    /// Glyphs of one font on one line of text.
    GlyphRun {
        /// Where the line starts on its baseline, on the surface.
        origin: Point,
        font: FontId,
        /// The glyphs of the run, see [`DisplayList::glyphs`](crate::DisplayList::glyphs);
        /// their positions are relative to `origin`.
        glyphs: ItemRange,
        color: Color,
    },
    /// The line of a text decoration: an underline or a line through the text.
    TextDecoration {
        rect: Rect,
        color: Color,
    },
    /// A rectangle of one color: the highlight of selected text, a caret.
    Fill {
        rect: Rect,
        color: Color,
    },
    /// Clips the items up to the matching [`DisplayItem::PopClip`] to the padding box of the frame.
    PushClip {
        frame: BoxFrame,
    },
    PopClip,
    /// Transforms and composites the items up to the matching [`DisplayItem::PopGroup`] as one.
    PushGroup {
        effects: GroupEffects,
    },
    PopGroup,
    /// The picture of an image, placed by its `object-fit` and cut to the box of the element.
    Image {
        image: ImageId,
        /// Where the whole picture lies; it may reach past `clip`.
        dest: Rect,
        /// The content box of the element: only what lies inside is drawn.
        clip: Rect,
    },
    /// An icon rasterized to fill `dest`, tinted with `color` unless it has colors of its own.
    Icon {
        icon: IconId,
        dest: Rect,
        color: Color,
    },
    /// A region the application draws itself, cut to `dest`.
    Canvas {
        canvas: CanvasId,
        dest: Rect,
    },
}
