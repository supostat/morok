use morok_core::{AccessibilityAction, Key, KeyEvent, KeyPhase, PointerEventKind, Propagation};
use morok_reactive::Runtime;

use crate::view::{ElementEvent, Property, Surfaces};

/// The keys that activate a control while it has focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActivationKeys {
    /// Space, as for a check box, a switch or a radio button.
    Space,
    /// Space and Return, as for a push button.
    SpaceAndReturn,
}

impl ActivationKeys {
    fn contain(self, key: Key) -> bool {
        match self {
            Self::Space => key == Key::Space,
            Self::SpaceAndReturn => matches!(key, Key::Space | Key::Enter),
        }
    }
}

/// Hands the event to `activate` when it activates the control: a click, an activation key
/// going down with no modifier held, or an assistive technology asking for the default action.
/// A key held down activates once, and a disabled control is activated by nothing; either way
/// the event is used.
pub(crate) fn handle_activation(
    runtime: &mut Runtime<Surfaces>,
    event: &ElementEvent<'_>,
    keys: ActivationKeys,
    is_disabled: &Property<bool>,
    activate: &mut dyn FnMut(&mut Runtime<Surfaces>),
) -> Propagation {
    if !activates(event, keys) {
        return Propagation::Continue;
    }
    let is_held_down = matches!(event, ElementEvent::Key(key) if key.is_repeat);
    if !is_held_down && !is_disabled.get(runtime) {
        activate(runtime);
    }
    Propagation::Stop
}

fn activates(event: &ElementEvent<'_>, keys: ActivationKeys) -> bool {
    match event {
        ElementEvent::Pointer(pointer) => matches!(pointer.kind, PointerEventKind::Clicked { .. }),
        ElementEvent::Key(key) => is_press_of(key, keys),
        ElementEvent::Accessibility(action) => *action == AccessibilityAction::Click,
        ElementEvent::TextChanged(_) => false,
    }
}

fn is_press_of(key: &KeyEvent, keys: ActivationKeys) -> bool {
    key.phase == KeyPhase::Pressed && key.modifiers.is_empty() && keys.contain(key.key)
}
