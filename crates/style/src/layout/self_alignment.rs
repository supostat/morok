use crate::layout::item_alignment::ItemAlignment;

/// A value of `justify-self` or `align-self`: the alignment the container gives, or the item's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelfAlignment {
    Auto,
    Item(ItemAlignment),
}
