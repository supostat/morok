use crate::kit_icon::KitIcon;
use crate::view::Property;

/// A place a sidebar leads to: its name, with an icon before it and a detail after it when it
/// has them.
#[derive(Clone, Debug)]
pub struct SidebarItem<Value> {
    pub(crate) value: Value,
    pub(crate) label: Property<String>,
    pub(crate) icon: Option<KitIcon>,
    pub(crate) detail: Option<Property<String>>,
}

impl<Value> SidebarItem<Value> {
    /// An item that selects `value`.
    pub fn new(value: Value, label: impl Into<Property<String>>) -> Self {
        Self {
            value,
            label: label.into(),
            icon: None,
            detail: None,
        }
    }

    /// Shows an icon of the kit before the name.
    #[must_use]
    pub fn icon(mut self, icon: KitIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Shows a short note at the end of the item: a count, a size.
    #[must_use]
    pub fn detail(mut self, detail: impl Into<Property<String>>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}
