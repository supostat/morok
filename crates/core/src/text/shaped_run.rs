use crate::paint::item_range::ItemRange;
use crate::text::font_id::FontId;

/// Consecutive glyphs of a line set in one font.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapedRun {
    pub font: FontId,
    /// The glyphs of the run in the glyph buffer it was shaped into.
    pub glyphs: ItemRange,
}
