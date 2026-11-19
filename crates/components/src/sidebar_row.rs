use crate::sidebar_item::SidebarItem;
use crate::view::Property;

/// A line of a sidebar: the heading of a group of items, or an item.
#[derive(Clone, Debug)]
pub(crate) enum SidebarRow<Value> {
    Heading(Property<String>),
    Item(SidebarItem<Value>),
}
