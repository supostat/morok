use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{AccessibleActions, AccessibleItem, ElementId, Focusability};
use morok_reactive::{Runtime, Signal};
use morok_style::States;

use crate::chosen_mark::ChosenMark;
use crate::dropdown::Dropdown;
use crate::kit_icon::KitIcon;
use crate::select_input::SelectInput;
use crate::styles::CLASSES;
use crate::type_ahead::TypeAhead;
use crate::view::{Mount, Property, Surfaces};

/// A box that holds one of several options and drops a list of them down under itself: it
/// shows the option whose value the signal holds, and choosing a row of the list writes the
/// value of that row. A click, Space, Return and the vertical arrows open the list; the
/// arrows, Home and End move its highlight, a click and Return choose, and Escape or a press
/// outside close it. Typing the start of a name goes to the option of that name: it is chosen
/// while the list is closed and highlighted while it is open.
pub struct Select<Value> {
    selected: Signal<Value>,
    options: Vec<(Value, String)>,
    label: Property<String>,
    is_disabled: Property<bool>,
}

impl<Value: Clone + PartialEq + 'static> Select<Value> {
    /// A select with no options yet.
    pub fn new(selected: Signal<Value>) -> Self {
        Self {
            selected,
            options: Vec::new(),
            label: Property::Fixed(String::new()),
            is_disabled: Property::Fixed(false),
        }
    }

    /// Adds an option that chooses `value`, after the ones added before.
    #[must_use]
    pub fn option(mut self, value: Value, name: impl Into<String>) -> Self {
        self.options.push((value, name.into()));
        self
    }

    /// What an assistive technology calls the select.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// A disabled select takes no focus and does not open.
    #[must_use]
    pub fn disabled(mut self, is_disabled: impl Into<Property<bool>>) -> Self {
        self.is_disabled = is_disabled.into();
        self
    }

    /// Builds the select: a box that holds the name of the chosen option and a chevron. Its
    /// list opens in an overlay of its own, as wide as the box.
    ///
    /// # Panics
    ///
    /// When the select has no options.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self {
            selected,
            options,
            label,
            is_disabled,
        } = self;
        let dropdown = dropdown_of(mount.runtime(), selected, options);
        let (names, chosen, is_open) = (Rc::clone(&dropdown.labels), dropdown.chosen, dropdown.is_open);
        let runtime = mount.runtime();
        let chosen_name = runtime.memo(move |runtime| {
            runtime
                .get(chosen)
                .map_or_else(String::new, |option| names[option].clone())
        });
        let list_state = runtime.memo(move |runtime| AccessibleItem {
            expanded: Some(runtime.get(is_open)),
            ..AccessibleItem::default()
        });
        let name_text = mount.text(chosen_name).class(CLASSES.select_name).id();
        let chevron = mount.kit_icon(KitIcon::Down).class(CLASSES.select_chevron).id();
        let select = mount
            .element()
            .class(CLASSES.select)
            .state(States::DISABLED, is_disabled.clone())
            .focusable(Focusability::TabStop)
            .accessible_role(AccessibleRole::ComboBox)
            .accessible_name(label.clone())
            .accessible_text(chosen_name)
            .accessible_item(list_state)
            .accessible_actions(AccessibleActions::CLICK)
            .child(name_text)
            .child(chevron)
            .id();
        let input = SelectInput {
            dropdown: dropdown.clone(),
            type_ahead: TypeAhead::mount(mount, select),
            is_disabled,
        };
        mount
            .build(select)
            .on_event(move |runtime, event| input.event(runtime, event));
        dropdown.attach(mount, select, label);
        select
    }

    /// Builds the panel of the list on the surface of `mount` itself, for a host that shows
    /// the panel in a place of its own. Choosing a row writes the signal as it does in an
    /// overlay.
    ///
    /// # Panics
    ///
    /// When the select has no options.
    pub fn mount_panel(self, mount: &mut Mount<'_>) -> ElementId {
        let dropdown = dropdown_of(mount.runtime(), self.selected, self.options);
        dropdown.mount_panel(mount, self.label)
    }
}

/// The list of the options: every one of them, the one whose value the signal holds marked
/// with a check.
fn dropdown_of<Value: Clone + PartialEq + 'static>(
    runtime: &mut Runtime<Surfaces>,
    selected: Signal<Value>,
    options: Vec<(Value, String)>,
) -> Dropdown {
    assert!(!options.is_empty(), "a select has options");
    let (values, names): (Vec<Value>, Vec<String>) = options.into_iter().unzip();
    let values: Rc<[Value]> = values.into();
    let shown = runtime.signal((0..values.len()).collect::<Vec<usize>>());
    let held = Rc::clone(&values);
    let chosen =
        runtime.memo(move |runtime| runtime.with(selected, |selected| held.iter().position(|value| value == selected)));
    Dropdown::new(
        runtime,
        names.into(),
        shown.into(),
        chosen.into(),
        ChosenMark::CheckMark,
        Rc::new(move |runtime, option| runtime.set(selected, values[option].clone())),
    )
}

impl<Value: Debug> Debug for Select<Value> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("Select")
            .field("selected", &self.selected)
            .field("options", &self.options)
            .field("label", &self.label)
            .field("is_disabled", &self.is_disabled)
            .finish()
    }
}
