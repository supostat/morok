use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{
    AccessibilityAction, AccessibleActions, AccessibleItem, ElementId, Focusability, Key, KeyEvent, KeyPhase,
    Modifiers, PointerEventKind, Propagation,
};
use morok_reactive::{Runtime, Signal};
use morok_style::States;

use crate::styles::CLASSES;
use crate::view::{ElementEvent, Mount, Property, Surfaces};
use crate::virtual_rows::VirtualRows;

/// How high a row of a list is.
const ROW_HEIGHT: f32 = 28.0;

/// How many rows of a list the user selects at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListSelection {
    /// One row: a click or an arrow key selects the row it lands on.
    Single,
    /// Any rows: Command-click adds or takes away a row, Shift-click and Shift with an arrow
    /// key select the rows from where the selection began, Space adds or takes away the row
    /// the keyboard is on.
    Multiple,
}

/// Rows of text the user selects from, as many as there may be: only the rows the viewport
/// shows have elements. The arrow keys move through the rows, Home and End to the ends, and
/// the list scrolls to keep the row the keyboard is on in view.
pub struct List {
    count: Property<usize>,
    label_of: Rc<dyn Fn(usize) -> String>,
    selected: Signal<Vec<usize>>,
    selection: ListSelection,
    label: Property<String>,
}

impl List {
    /// A list of `count` rows, each the text `label_of` gives for its index, whose selected
    /// rows are the indices `selected` holds, in ascending order. One row is selected at once.
    pub fn new(
        count: impl Into<Property<usize>>,
        label_of: impl Fn(usize) -> String + 'static,
        selected: Signal<Vec<usize>>,
    ) -> Self {
        Self {
            count: count.into(),
            label_of: Rc::new(label_of),
            selected,
            selection: ListSelection::Single,
            label: Property::Fixed(String::new()),
        }
    }

    #[must_use]
    pub fn selection(mut self, selection: ListSelection) -> Self {
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
        let choice = RowChoice {
            selected,
            selection,
            keyboard_row: runtime.signal(0),
            anchor: runtime.signal(0),
        };
        let rows = VirtualRows::mount(mount, count.into(), ROW_HEIGHT, move |mount, index| {
            let runtime = mount.runtime();
            let label_of = Rc::clone(&label_of);
            let text = runtime.memo(move |runtime| label_of(runtime.get(index)));
            let is_selected = runtime.memo(move |runtime| {
                let index = runtime.get(index);
                runtime.with(selected, |selected| selected.binary_search(&index).is_ok())
            });
            let item = runtime.memo(move |runtime| AccessibleItem {
                position_in_set: Some(runtime.get(index) + 1),
                size_of_set: Some(runtime.get(count)),
                ..AccessibleItem::default()
            });
            let label_text = mount.text(text).id();
            mount
                .element()
                .class(CLASSES.list_row)
                .state(States::CHECKED, is_selected)
                .accessible_role(AccessibleRole::ListBoxOption)
                .accessible_name(text)
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
                .id()
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

/// What the user does to the selection of a list.
#[derive(Clone, Copy, Debug)]
struct RowChoice {
    selected: Signal<Vec<usize>>,
    selection: ListSelection,
    /// The row the keyboard is on.
    keyboard_row: Signal<usize>,
    /// Where a selection by Shift begins.
    anchor: Signal<usize>,
}

impl RowChoice {
    fn click(self, runtime: &mut Runtime<Surfaces>, index: usize, modifiers: Modifiers) {
        let extends = self.selection == ListSelection::Multiple;
        if extends && modifiers.contains(Modifiers::SHIFT) {
            self.select_from_anchor(runtime, index);
        } else if extends && modifiers.contains(Modifiers::COMMAND) {
            self.toggle(runtime, index);
            runtime.set(self.anchor, index);
        } else {
            self.select_only(runtime, index);
        }
        runtime.set(self.keyboard_row, index);
    }

    fn key(self, runtime: &mut Runtime<Surfaces>, key: &KeyEvent, count: usize) -> Propagation {
        if key.phase != KeyPhase::Pressed || count == 0 {
            return Propagation::Continue;
        }
        let now = runtime.get(self.keyboard_row).min(count - 1);
        let extends = self.selection == ListSelection::Multiple && key.modifiers == Modifiers::SHIFT;
        let moved_to = match (key.key, key.modifiers) {
            (Key::ArrowDown, Modifiers::NONE | Modifiers::SHIFT) => (now + 1).min(count - 1),
            (Key::ArrowUp, Modifiers::NONE | Modifiers::SHIFT) => now.saturating_sub(1),
            (Key::Home, Modifiers::NONE | Modifiers::SHIFT) => 0,
            (Key::End, Modifiers::NONE | Modifiers::SHIFT) => count - 1,
            (Key::Space, Modifiers::NONE) => {
                match self.selection {
                    ListSelection::Single => self.select_only(runtime, now),
                    ListSelection::Multiple => self.toggle(runtime, now),
                }
                return Propagation::Stop;
            }
            _ => return Propagation::Continue,
        };
        if extends {
            self.select_from_anchor(runtime, moved_to);
        } else {
            self.select_only(runtime, moved_to);
        }
        runtime.set(self.keyboard_row, moved_to);
        Propagation::Stop
    }

    fn select_only(self, runtime: &mut Runtime<Surfaces>, index: usize) {
        runtime.set(self.selected, vec![index]);
        runtime.set(self.anchor, index);
    }

    fn select_from_anchor(self, runtime: &mut Runtime<Surfaces>, index: usize) {
        let anchor = runtime.get(self.anchor);
        runtime.set(self.selected, (anchor.min(index)..=anchor.max(index)).collect());
    }

    fn toggle(self, runtime: &mut Runtime<Surfaces>, index: usize) {
        runtime.update(self.selected, |selected| match selected.binary_search(&index) {
            Ok(position) => {
                selected.remove(position);
            }
            Err(position) => selected.insert(position, index),
        });
    }
}
