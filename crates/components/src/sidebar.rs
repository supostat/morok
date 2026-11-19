use std::fmt::{Debug, Formatter, Result};

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{AccessibleActions, ElementId};
use morok_reactive::Signal;
use morok_style::States;

use crate::sidebar_item::SidebarItem;
use crate::sidebar_row::SidebarRow;
use crate::single_choice::SingleChoice;
use crate::styles::CLASSES;
use crate::view::{Mount, Property};

/// The column at the side of a window that leads to its places: items in groups under
/// headings, of which one is selected — the one whose value the signal holds. A click, Space
/// while an item has focus and an assistive technology select an item; the arrow keys select
/// the next or the previous one. The Tab key stops on the selected item alone.
pub struct Sidebar<Value> {
    selected: Signal<Value>,
    rows: Vec<SidebarRow<Value>>,
    label: Property<String>,
}

impl<Value: Clone + PartialEq + 'static> Sidebar<Value> {
    /// A sidebar with no items yet.
    pub fn new(selected: Signal<Value>) -> Self {
        Self {
            selected,
            rows: Vec::new(),
            label: Property::Fixed(String::new()),
        }
    }

    /// Adds the heading of the items that follow.
    #[must_use]
    pub fn heading(mut self, heading: impl Into<Property<String>>) -> Self {
        self.rows.push(SidebarRow::Heading(heading.into()));
        self
    }

    /// Adds an item after the rows added before.
    #[must_use]
    pub fn item(mut self, item: SidebarItem<Value>) -> Self {
        self.rows.push(SidebarRow::Item(item));
        self
    }

    /// What an assistive technology calls the sidebar.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// Builds the sidebar: a column of its headings and items.
    ///
    /// # Panics
    ///
    /// When the sidebar has no items.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self { selected, rows, label } = self;
        let values = rows
            .iter()
            .filter_map(|row| match row {
                SidebarRow::Item(item) => Some(item.value.clone()),
                SidebarRow::Heading(_) => None,
            })
            .collect();
        let choice = SingleChoice::new(mount.runtime(), selected, values, Property::Fixed(false));
        let mut row_elements = Vec::with_capacity(rows.len());
        let mut item_elements = Vec::new();
        for row in rows {
            match row {
                SidebarRow::Heading(heading) => {
                    row_elements.push(mount.text(heading).class(CLASSES.sidebar_heading).id());
                }
                SidebarRow::Item(item) => {
                    let element = mount_item(mount, &choice, item_elements.len(), item);
                    row_elements.push(element);
                    item_elements.push(element);
                }
            }
        }
        choice.focus_follows_choice(mount, item_elements);
        let mut sidebar = mount.element();
        sidebar
            .class(CLASSES.sidebar)
            .accessible_role(AccessibleRole::ListBox)
            .accessible_name(label);
        for row in row_elements {
            sidebar.child(row);
        }
        sidebar.id()
    }
}

/// Builds the item at `position` among the items: its icon, its name and its detail.
fn mount_item<Value: Clone + PartialEq + 'static>(
    mount: &mut Mount<'_>,
    choice: &SingleChoice<Value>,
    position: usize,
    item: SidebarItem<Value>,
) -> ElementId {
    let icon = item
        .icon
        .map(|icon| mount.kit_icon(icon).class(CLASSES.sidebar_item_icon).id());
    let name = mount.text(item.label.clone()).id();
    let detail = item
        .detail
        .map(|detail| mount.text(detail).class(CLASSES.sidebar_item_detail).id());
    let is_chosen = choice.is_chosen(mount.runtime(), position);
    let focusability = choice.focusability(mount.runtime(), position);
    let mut element = mount.element();
    element
        .class(CLASSES.sidebar_item)
        .state(States::CHECKED, is_chosen)
        .focusable(focusability)
        .accessible_role(AccessibleRole::ListBoxOption)
        .accessible_name(item.label)
        .accessible_actions(AccessibleActions::CLICK)
        .on_event(choice.handler(position));
    for part in icon.into_iter().chain([name]).chain(detail) {
        element.child(part);
    }
    element.id()
}

impl<Value: Debug> Debug for Sidebar<Value> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("Sidebar")
            .field("selected", &self.selected)
            .field("rows", &self.rows)
            .field("label", &self.label)
            .finish()
    }
}
