use crate::paint::display_item::DisplayItem;
use crate::paint::filter::Filter;
use crate::paint::gradient_fill::GradientFill;
use crate::paint::gradient_stop::GradientStop;
use crate::paint::group_effects::GroupEffects;
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

    /// Appends an item of `source` with the entries of the buffers of `source` it points
    /// into: the stops of a gradient, the glyphs of a run and the filters of a group move
    /// with it.
    pub(crate) fn push_copy(&mut self, item: DisplayItem, source: &Self) {
        let item = match item {
            DisplayItem::Gradient { frame, gradient } => DisplayItem::Gradient {
                frame,
                gradient: GradientFill {
                    shape: gradient.shape,
                    stops: appended(&mut self.gradient_stops, source.gradient_stops(gradient.stops)),
                },
            },
            DisplayItem::GlyphRun {
                origin,
                font,
                glyphs,
                color,
            } => DisplayItem::GlyphRun {
                origin,
                font,
                glyphs: appended(&mut self.glyphs, source.glyphs(glyphs)),
                color,
            },
            DisplayItem::PushGroup { effects } => DisplayItem::PushGroup {
                effects: GroupEffects {
                    filter: appended(&mut self.filters, source.filters(effects.filter)),
                    backdrop_filter: appended(&mut self.filters, source.filters(effects.backdrop_filter)),
                    ..effects
                },
            },
            other => other,
        };
        self.items.push(item);
    }

    pub(crate) fn clear(&mut self) {
        self.items.clear();
        self.gradient_stops.clear();
        self.glyphs.clear();
        self.filters.clear();
    }
}

/// Appends the entries to the buffer and returns where they are in it.
fn appended<Entry: Clone>(buffer: &mut Vec<Entry>, entries: &[Entry]) -> ItemRange {
    let start = buffer.len() as u32;
    buffer.extend_from_slice(entries);
    ItemRange {
        start,
        end: buffer.len() as u32,
    }
}
