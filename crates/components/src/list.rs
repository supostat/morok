use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{
    AccessibilityAction, AccessibleActions, AccessibleItem, ElementId, Focusability, Modifiers, PointerEventKind,
    Propagation,
};
use morok_reactive::Signal;
use morok_style::States;

use crate::row_choice::RowChoice;
use crate::row_selection::RowSelection;
use crate::styles::CLASSES;
use crate::view::{ElementEvent, Mount, Property};
use crate::virtual_rows::VirtualRows;

/// How high a row of a list is.
const ROW_HEIGHT: f32 = 28.0;

/// Writes the text of the row of the given index.
type RowLabel = dyn Fn(usize, &mut String);

/// Rows of text the user selects from, as many as there may be: only the rows the viewport
/// shows have elements, and a scroll allocates nothing. The arrow keys move through the rows,
/// Home and End to the ends, and the list scrolls to keep the row the keyboard is on in view.
pub struct List {
    count: Property<usize>,
    label_of: Rc<RowLabel>,
    selected: Signal<Vec<usize>>,
    selection: RowSelection,
    label: Property<String>,
}

impl List {
    /// A list of `count` rows, each the text `label_of` writes for its index into the string
    /// it is given, empty, whose selected rows are the indices `selected` holds, in ascending
    /// order. One row is selected at once.
    pub fn new(
        count: impl Into<Property<usize>>,
        label_of: impl Fn(usize, &mut String) + 'static,
        selected: Signal<Vec<usize>>,
    ) -> Self {
        Self {
            count: count.into(),
            label_of: Rc::new(label_of),
            selected,
            selection: RowSelection::Single,
            label: Property::Fixed(String::new()),
        }
    }

    #[must_use]
    pub fn selection(mut self, selection: RowSelection) -> Self {
        self.selection = selection;
        self
    }

    /// What an assistive technology calls the list.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// Builds the list: a view whose content scrolls, with a row slot for each row in view.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self {
            count,
            label_of,
            selected,
            selection,
            label,
        } = self;
        let runtime = mount.runtime();
        let count = runtime.memo(move |runtime| count.get(runtime));
        let choice = RowChoice::new(runtime, selected, selection);
        let rows = VirtualRows::mount(mount, count.into(), ROW_HEIGHT, move |mount, index| {
            let runtime = mount.runtime();
            let is_selected = choice.is_selected(runtime, index);
            let item = runtime.memo(move |runtime| AccessibleItem {
                position_in_set: Some(runtime.get(index) + 1),
                size_of_set: Some(runtime.get(count)),
                ..AccessibleItem::default()
            });
            let label_text = mount.text("").class(CLASSES.list_row_label).id();
            let row = mount
                .element()
                .class(CLASSES.list_row)
                .state(States::CHECKED, is_selected)
                .accessible_role(AccessibleRole::ListBoxOption)
                .accessible_item(item)
                .accessible_actions(AccessibleActions::CLICK)
                .on_event(move |runtime, event| {
                    let index = runtime.get(index);
                    match event {
                        ElementEvent::Pointer(pointer) if matches!(pointer.kind, PointerEventKind::Clicked { .. }) => {
                            choice.click(runtime, index, pointer.modifiers);
                            Propagation::Stop
                        }
                        ElementEvent::Accessibility(AccessibilityAction::Click) => {
                            choice.click(runtime, index, Modifiers::NONE);
                            Propagation::Stop
                        }
                        _ => Propagation::Continue,
                    }
                })
                .child(label_text)
                .id();
            let label_of = Rc::clone(&label_of);
            let mut label = String::new();
            mount.effect(move |runtime, tree| {
                label.clear();
                label_of(runtime.get(index), &mut label);
                tree.set_text(label_text, &label);
                tree.set_accessible_name(row, &label);
            });
            row
        });
        mount
            .build(rows.frame.viewport)
            .class(CLASSES.list)
            .focusable(Focusability::TabStop)
            .accessible_role(AccessibleRole::ListBox)
            .accessible_name(label)
            .on_event(move |runtime, event| match event {
                ElementEvent::Key(key) => {
                    let count = runtime.get(count);
                    choice.key(runtime, key, count)
                }
                _ => Propagation::Continue,
            });
        rows.keep_in_view(mount, choice.keyboard_row);
        rows.frame.view
    }
}

impl Debug for List {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("List")
            .field("count", &self.count)
            .field("selected", &self.selected)
            .field("selection", &self.selection)
            .field("label", &self.label)
            .finish_non_exhaustive()
    }
}
