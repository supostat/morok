use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use morok_core::accesskit::{Live, Role as AccessibleRole};
use morok_core::{AccessibleItem, ElementId, Key, KeyEvent, KeyPhase, PointerEventKind, Propagation};
use morok_reactive::{Memo, Runtime, Signal};
use morok_style::{Declaration, Position, States};

use crate::dropdown::Dropdown;
use crate::dropdown_step::DropdownStep;
use crate::field_elements::FieldElements;
use crate::kit_icon::KitIcon;
use crate::styles::CLASSES;
use crate::text_field_parts::{FieldKind, FieldParts};
use crate::view::{ElementEvent, Mount, Property, Surfaces};

/// A text field with a list of options that drops down under it. What the user types is
/// written to the signal the field edits, as in a text field, and the list shows the options
/// whose names hold what was typed, whatever the case. The chevron opens the list with every
/// option in it until the text is edited again. The vertical arrows open the list and move
/// its highlight; Return on the highlighted row and a click on a row write the name of the
/// option to the signal; Escape, a press outside and the chevron close the list. The text is
/// the user's: it need not name an option. An assistive technology is told the name of the
/// highlighted option and its place in the list as the highlight moves.
pub struct Combobox {
    parts: FieldParts,
    options: Vec<String>,
}

impl Combobox {
    /// A combo box with no options yet.
    pub fn new(value: Signal<String>) -> Self {
        Self {
            parts: FieldParts::new(value),
            options: Vec::new(),
        }
    }

    /// Adds an option of the name, after the ones added before.
    #[must_use]
    pub fn option(mut self, name: impl Into<String>) -> Self {
        self.options.push(name.into());
        self
    }

    /// What the field shows while it is empty.
    #[must_use]
    pub fn placeholder(mut self, placeholder: impl Into<Property<String>>) -> Self {
        self.parts.placeholder = placeholder.into();
        self
    }

    /// What an assistive technology calls the combo box.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.parts.label = label.into();
        self
    }

    /// A disabled combo box takes no focus and does not open.
    #[must_use]
    pub fn disabled(mut self, is_disabled: impl Into<Property<bool>>) -> Self {
        self.parts.is_disabled = is_disabled.into();
        self
    }

    /// Shows that what the field holds is not valid.
    #[must_use]
    pub fn invalid(mut self, is_invalid: impl Into<Property<bool>>) -> Self {
        self.parts.is_invalid = is_invalid.into();
        self
    }

    /// Names what Return does in the field while no row of the list is highlighted.
    #[must_use]
    pub fn on_submit(mut self, handler: impl FnMut(&mut Runtime<Surfaces>) + 'static) -> Self {
        self.parts.on_submit = Some(Box::new(handler));
        self
    }

    /// Builds the combo box: the field of the kit with a chevron after its line. Its list
    /// opens in an overlay of its own, as wide as the field.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self { mut parts, options } = self;
        let (label, is_disabled) = (parts.label.clone(), parts.is_disabled.clone());
        let shows_every_option = mount.runtime().signal(false);
        let dropdown = dropdown_of(mount.runtime(), parts.value, options, shows_every_option);
        let keys = dropdown.clone();
        parts.on_key = Some(Box::new(move |runtime, key| list_key(&keys, runtime, key)));
        let edited = dropdown.clone();
        parts.on_edit = Some(Box::new(move |runtime| {
            if runtime.get(shows_every_option) {
                runtime.set(shows_every_option, false);
            }
            edited.open(runtime, None);
        }));
        let FieldElements { field, input } = parts.mount(FieldKind::Line, mount);

        let (is_open, is_shown) = (dropdown.is_open, dropdown.is_shown);
        let list_state = mount.runtime().memo(move |runtime| AccessibleItem {
            expanded: Some(runtime.get(is_shown)),
            ..AccessibleItem::default()
        });
        mount
            .build(input)
            .accessible_role(AccessibleRole::EditableComboBox)
            .accessible_item(list_state);
        // The keys of an open list are the field's: a list the chevron opened puts focus there.
        mount.effect(move |runtime, tree| {
            if runtime.get(is_open) {
                tree.focus(input);
            }
        });
        let toggled = dropdown.clone();
        let chevron_is_disabled = is_disabled.clone();
        let chevron = mount
            .kit_icon(KitIcon::Down)
            .class(CLASSES.combobox_chevron)
            .state(States::DISABLED, is_disabled)
            .on_event(move |runtime, event| match event {
                ElementEvent::Pointer(pointer) if matches!(pointer.kind, PointerEventKind::Clicked { .. }) => {
                    if !chevron_is_disabled.get(runtime) {
                        toggle(&toggled, shows_every_option, runtime);
                    }
                    Propagation::Stop
                }
                _ => Propagation::Continue,
            })
            .id();
        // The list is an overlay with an accessibility tree of its own: the field's tree
        // speaks its highlight.
        let spoken = spoken_highlight(mount.runtime(), &dropdown);
        let voice = mount
            .element()
            .inline(Declaration::Position(Position::Absolute))
            .accessible_role(AccessibleRole::Status)
            .accessible_live(Live::Polite)
            .accessible_name(spoken)
            .id();
        mount.build(field).child(chevron).child(voice);
        dropdown.attach(mount, field, label);
        field
    }

    /// Builds the panel of the list on the surface of `mount` itself, for a host that shows
    /// the panel in a place of its own: the rows of the options the text of the signal leaves.
    /// Choosing a row writes the signal as it does in an overlay.
    pub fn mount_panel(self, mount: &mut Mount<'_>) -> ElementId {
        let shows_every_option = mount.runtime().signal(false);
        let dropdown = dropdown_of(mount.runtime(), self.parts.value, self.options, shows_every_option);
        dropdown.mount_panel(mount, self.parts.label)
    }
}

/// The list of the options whose names hold the text of the signal, whatever the case, or of
/// every option while `shows_every_option` holds; the option the text names in full is marked.
fn dropdown_of(
    runtime: &mut Runtime<Surfaces>,
    value: Signal<String>,
    options: Vec<String>,
    shows_every_option: Signal<bool>,
) -> Dropdown {
    let names: Rc<[String]> = options.into();
    let names_in_lower_case: Vec<String> = names.iter().map(|name| name.to_lowercase()).collect();
    let shown = runtime.memo(move |runtime| {
        if runtime.get(shows_every_option) {
            return (0..names_in_lower_case.len()).collect::<Vec<usize>>();
        }
        let sought = runtime.with(value, |value| value.to_lowercase());
        (0..names_in_lower_case.len())
            .filter(|option| names_in_lower_case[*option].contains(&sought))
            .collect::<Vec<usize>>()
    });
    let held = Rc::clone(&names);
    let chosen = runtime.memo(move |runtime| runtime.with(value, |value| held.iter().position(|name| name == value)));
    let written = Rc::clone(&names);
    Dropdown::new(
        runtime,
        names,
        shown.into(),
        chosen.into(),
        Rc::new(move |runtime, option| runtime.set(value, written[option].clone())),
    )
}

/// What the chevron does: it closes a list that is on screen and opens one that is not, with
/// every option in it and the highlight on the option the text names.
fn toggle(dropdown: &Dropdown, shows_every_option: Signal<bool>, runtime: &mut Runtime<Surfaces>) {
    if runtime.get(dropdown.is_shown) {
        dropdown.close(runtime);
    } else {
        runtime.set(shows_every_option, true);
        let chosen = dropdown.row_of_chosen(runtime);
        dropdown.open(runtime, chosen);
    }
}

/// What the field says of the highlighted row while the list is on screen: the name of its
/// option and its place among the rows, as "Movies, 3 of 12"; nothing while no row is.
fn spoken_highlight(runtime: &mut Runtime<Surfaces>, dropdown: &Dropdown) -> Memo<String> {
    let (is_shown, highlighted, shown) = (dropdown.is_shown, dropdown.highlighted, dropdown.shown);
    let labels = Rc::clone(&dropdown.labels);
    runtime.memo(move |runtime| {
        if !runtime.get(is_shown) {
            return String::new();
        }
        let Some(row) = runtime.get(highlighted) else {
            return String::new();
        };
        runtime.with(shown, |shown| {
            shown.get(row).map_or_else(String::new, |option| {
                format!("{}, {} of {}", labels[*option], row + 1, shown.len())
            })
        })
    })
}

/// What a key does to the list while the keyboard is in the field: the vertical arrows open
/// the list and move its highlight, Return chooses the highlighted row, Escape closes the
/// list, and Tab closes it on its way. Every other key is the text's.
fn list_key(dropdown: &Dropdown, runtime: &mut Runtime<Surfaces>, key: &KeyEvent) -> Propagation {
    if key.phase != KeyPhase::Pressed || !key.modifiers.is_empty() {
        return Propagation::Continue;
    }
    let is_shown = runtime.get(dropdown.is_shown);
    match key.key {
        Key::ArrowDown | Key::ArrowUp => {
            if !is_shown {
                dropdown.open(runtime, None);
            }
            let step = if key.key == Key::ArrowDown {
                DropdownStep::Next
            } else {
                DropdownStep::Previous
            };
            dropdown.move_highlight(runtime, step);
            Propagation::Stop
        }
        Key::Enter if is_shown => {
            if dropdown.choose_highlighted(runtime) {
                Propagation::Stop
            } else {
                dropdown.close(runtime);
                Propagation::Continue
            }
        }
        Key::Escape if is_shown => {
            dropdown.close(runtime);
            Propagation::Stop
        }
        Key::Tab => {
            dropdown.close(runtime);
            Propagation::Continue
        }
        _ => Propagation::Continue,
    }
}

impl Debug for Combobox {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("Combobox")
            .field("options", &self.options)
            .field("placeholder", &self.parts.placeholder)
            .field("is_disabled", &self.parts.is_disabled)
            .field("is_invalid", &self.parts.is_invalid)
            .finish_non_exhaustive()
    }
}
