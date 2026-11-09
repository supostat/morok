use crate::paint::display_item::DisplayItem;
use crate::paint::gradient_stop::GradientStop;
use crate::paint::item_range::ItemRange;

/// Everything a surface draws, in paint order.
#[derive(Debug, Default)]
pub struct DisplayList {
    pub(crate) items: Vec<DisplayItem>,
    pub(crate) gradient_stops: Vec<GradientStop>,
}

impl DisplayList {
    pub fn items(&self) -> &[DisplayItem] {
        &self.items
    }

    /// The stops of one gradient of the list.
    pub fn gradient_stops(&self, stops: ItemRange) -> &[GradientStop] {
        &self.gradient_stops[stops.as_range()]
    }

    pub(crate) fn clear(&mut self) {
        self.items.clear();
        self.gradient_stops.clear();
    }
}
