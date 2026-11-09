use crate::paint::display_item::DisplayItem;
use crate::paint::filter::Filter;
use crate::paint::gradient_stop::GradientStop;
use crate::paint::item_range::ItemRange;
use crate::text::shaped_glyph::ShapedGlyph;

/// Everything a surface draws, in paint order.
#[derive(Debug, Default)]
pub struct DisplayList {
    pub(crate) items: Vec<DisplayItem>,
    pub(crate) gradient_stops: Vec<GradientStop>,
    pub(crate) glyphs: Vec<ShapedGlyph>,
    pub(crate) filters: Vec<Filter>,
}

impl DisplayList {
    pub fn items(&self) -> &[DisplayItem] {
        &self.items
    }

    /// The stops of one gradient of the list.
    pub fn gradient_stops(&self, stops: ItemRange) -> &[GradientStop] {
        &self.gradient_stops[stops.as_range()]
    }

    /// The glyphs of one glyph run of the list.
    pub fn glyphs(&self, glyphs: ItemRange) -> &[ShapedGlyph] {
        &self.glyphs[glyphs.as_range()]
    }

    /// The filters of one group of the list, in the order they apply.
    pub fn filters(&self, filters: ItemRange) -> &[Filter] {
        &self.filters[filters.as_range()]
    }

    pub(crate) fn clear(&mut self) {
        self.items.clear();
        self.gradient_stops.clear();
        self.glyphs.clear();
        self.filters.clear();
    }
}
