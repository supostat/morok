use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{AccessibilityAction, AccessibleActions, AccessibleItem, ElementId, PointerEventKind, Propagation};
use morok_reactive::Memo;
use morok_style::{Declaration, States};

use crate::dropdown::Dropdown;
use crate::kit_icon::KitIcon;
use crate::length_in_pixels::pixels;
use crate::styles::CLASSES;
use crate::view::{ElementEvent, Mount, Property};
use crate::virtual_rows::VirtualRows;

/// How high a row of the list is.
const ROW_HEIGHT: f32 = 26.0;
/// How many rows the list shows at once; past them it scrolls.
const MOST_ROWS_IN_VIEW: u8 = 8;

/// What an open dropdown shows: a floating panel around a row for each option the dropdown
/// shows, of which only the rows in view have elements. The row of the option the control
/// holds carries a check mark, and the highlighted row is the one a click or Return chooses.
pub(crate) struct OptionList {
    pub(crate) dropdown: Dropdown,
    /// What an assistive technology calls the list.
    pub(crate) label: Property<String>,
}

impl OptionList {
    pub(crate) fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self { dropdown, label } = self;
        let (shown, highlighted) = (dropdown.shown, dropdown.highlighted);
        let runtime = mount.runtime();
        let count = runtime.memo(move |runtime| runtime.with(shown, Vec::len));
        let rows_in_view = runtime.memo(move |runtime| {
            let in_view =
                u8::try_from(runtime.get(count)).map_or(MOST_ROWS_IN_VIEW, |count| count.min(MOST_ROWS_IN_VIEW));
            Declaration::Height(pixels(f32::from(in_view) * ROW_HEIGHT))
        });
        let highlighted_row = runtime.memo(move |runtime| runtime.get(highlighted).unwrap_or(0));
        let rows = VirtualRows::mount(mount, count.into(), ROW_HEIGHT, move |mount, row| {
            mount_row(mount, &dropdown, row, count)
        });
        mount.build(rows.frame.view).inline(rows_in_view);
        mount
            .build(rows.frame.viewport)
            .accessible_role(AccessibleRole::ListBox)
            .accessible_name(label);
        rows.keep_in_view(mount, highlighted_row);
        mount.element().class(CLASSES.options).child(rows.frame.view).id()
    }
}

/// Builds what a row slot holds: the check mark and the name of the option the row shows.
fn mount_row(mount: &mut Mount<'_>, dropdown: &Dropdown, row: Memo<usize>, count: Memo<usize>) -> ElementId {
    let (shown, chosen, highlighted) = (dropdown.shown, dropdown.chosen, dropdown.highlighted);
    let labels = dropdown.labels.clone();
    let runtime = mount.runtime();
    // A slot about to go with a list that shrank reads its row once more and finds no option.
    let option = runtime.memo(move |runtime| {
        let row = runtime.get(row);
        runtime.with(shown, |shown| shown.get(row).copied())
    });
    let name = runtime.memo(move |runtime| {
        runtime
            .get(option)
            .map_or_else(String::new, |option| labels[option].clone())
    });
    let is_chosen = runtime.memo(move |runtime| {
        let option = runtime.get(option);
        option.is_some() && option == runtime.get(chosen)
    });
    let is_highlighted = runtime.memo(move |runtime| runtime.get(highlighted) == Some(runtime.get(row)));
    let item = runtime.memo(move |runtime| AccessibleItem {
        position_in_set: Some(runtime.get(row) + 1),
        size_of_set: Some(runtime.get(count)),
        ..AccessibleItem::default()
    });
    let check_mark = mount.kit_icon(KitIcon::Check).class(CLASSES.option_mark).id();
    let name_text = mount.text(name).class(CLASSES.option_name).id();
    let events = dropdown.clone();
    mount
        .element()
        .class(CLASSES.option)
        .class_when(CLASSES.highlighted, is_highlighted)
        .state(States::CHECKED, is_chosen)
        .accessible_role(AccessibleRole::ListBoxOption)
        .accessible_name(name)
        .accessible_item(item)
        .accessible_actions(AccessibleActions::CLICK)
        .on_event(move |runtime, event| {
            let row = runtime.get(row);
            match event {
                ElementEvent::Pointer(pointer) => match pointer.kind {
                    // Only a pointer that moves takes the highlight: rows that scroll under a
                    // resting pointer leave it where the keyboard put it.
                    PointerEventKind::Moved => {
                        events.highlight(runtime, row);
                        Propagation::Continue
                    }
                    PointerEventKind::Clicked { .. } => {
                        events.choose_row(runtime, row);
                        Propagation::Stop
                    }
                    _ => Propagation::Continue,
                },
                ElementEvent::Accessibility(AccessibilityAction::Click) => {
                    events.choose_row(runtime, row);
                    Propagation::Stop
                }
                _ => Propagation::Continue,
            }
        })
        .child(check_mark)
        .child(name_text)
        .id()
}
