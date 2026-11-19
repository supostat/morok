use morok_core::{AccessibilityAction, Key, KeyEvent, KeyPhase, PointerEventKind, Propagation};
use morok_reactive::Runtime;

use crate::dropdown::Dropdown;
use crate::dropdown_step::DropdownStep;
use crate::type_ahead::TypeAhead;
use crate::view::{ElementEvent, Property, Surfaces};

/// What the pointer and the keys do to a select: they open and close its list, move the
/// highlight of the list and choose.
pub(crate) struct SelectInput {
    /// The list of every option of the select: a row shows the option of its own place.
    pub(crate) dropdown: Dropdown,
    pub(crate) type_ahead: TypeAhead,
    pub(crate) is_disabled: Property<bool>,
}

impl SelectInput {
    /// What the box of the select does with the events it hears of.
    pub(crate) fn event(&self, runtime: &mut Runtime<Surfaces>, event: &ElementEvent<'_>) -> Propagation {
        match event {
            ElementEvent::Pointer(pointer) if matches!(pointer.kind, PointerEventKind::Clicked { .. }) => {
                self.toggle(runtime);
                Propagation::Stop
            }
            ElementEvent::Accessibility(AccessibilityAction::Click) => {
                self.toggle(runtime);
                Propagation::Stop
            }
            ElementEvent::Key(key) => self.key(runtime, key),
            _ => Propagation::Continue,
        }
    }

    fn toggle(&self, runtime: &mut Runtime<Surfaces>) {
        if self.is_disabled.get(runtime) {
            return;
        }
        if runtime.get(self.dropdown.is_open) {
            self.dropdown.close(runtime);
        } else {
            let chosen = self.dropdown.row_of_chosen(runtime);
            self.dropdown.open(runtime, chosen);
        }
    }

    fn key(&self, runtime: &mut Runtime<Surfaces>, key: &KeyEvent) -> Propagation {
        if key.phase != KeyPhase::Pressed || self.is_disabled.get(runtime) {
            return Propagation::Continue;
        }
        if let Some(typed) = self.type_ahead.typed_by(runtime, key) {
            self.go_to_option_named(runtime, &typed);
            return Propagation::Stop;
        }
        if !key.modifiers.is_empty() {
            return Propagation::Continue;
        }
        if runtime.get(self.dropdown.is_open) {
            self.key_while_open(runtime, key)
        } else {
            self.key_while_closed(runtime, key)
        }
    }

    /// The keys that press the box open the list with the highlight on the chosen option, or
    /// on the first while none is chosen. A key held down opens it once.
    fn key_while_closed(&self, runtime: &mut Runtime<Surfaces>, key: &KeyEvent) -> Propagation {
        if !matches!(key.key, Key::Space | Key::Enter | Key::ArrowDown | Key::ArrowUp) {
            return Propagation::Continue;
        }
        if !key.is_repeat {
            let highlighted = self.dropdown.row_of_chosen(runtime).unwrap_or(0);
            self.dropdown.open(runtime, Some(highlighted));
        }
        Propagation::Stop
    }

    fn key_while_open(&self, runtime: &mut Runtime<Surfaces>, key: &KeyEvent) -> Propagation {
        match key.key {
            Key::ArrowDown => self.dropdown.move_highlight(runtime, DropdownStep::Next),
            Key::ArrowUp => self.dropdown.move_highlight(runtime, DropdownStep::Previous),
            Key::Home => self.dropdown.move_highlight(runtime, DropdownStep::First),
            Key::End => self.dropdown.move_highlight(runtime, DropdownStep::Last),
            // The press that opened the list, held down, does not choose from it.
            Key::Enter | Key::Space if key.is_repeat => {}
            Key::Enter | Key::Space => {
                if !self.dropdown.choose_highlighted(runtime) {
                    self.dropdown.close(runtime);
                }
            }
            Key::Escape => self.dropdown.close(runtime),
            // The list goes and focus moves on.
            Key::Tab => {
                self.dropdown.close(runtime);
                return Propagation::Continue;
            }
            _ => return Propagation::Continue,
        }
        Propagation::Stop
    }

    /// Goes to the first option whose name begins with what was typed, whatever the case:
    /// an open list highlights it, a closed one chooses it.
    fn go_to_option_named(&self, runtime: &mut Runtime<Surfaces>, typed: &str) {
        let named = self
            .dropdown
            .labels
            .iter()
            .position(|name| name.to_lowercase().starts_with(typed));
        let Some(option) = named else {
            return;
        };
        if runtime.get(self.dropdown.is_open) {
            self.dropdown.highlight(runtime, option);
        } else {
            self.dropdown.choose_row(runtime, option);
        }
    }
}
