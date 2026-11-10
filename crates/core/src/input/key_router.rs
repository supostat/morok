use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::input::input_handler::InputHandler;
use crate::input::key_event::KeyEvent;
use crate::input::propagation::Propagation;
use crate::input::shortcut::Shortcut;
use crate::input::shortcut_id::ShortcutId;
use crate::input::shortcut_scope::ShortcutScope;

/// Routes a key from the focused element upwards. Each element on the way is offered the key
/// first; when it does not use a press, the shortcuts registered on it are tried. The
/// shortcuts of the surface come last.
pub(crate) struct KeyRouter<'a> {
    pub(crate) elements: &'a ElementArena,
    pub(crate) shortcuts: &'a [Shortcut],
    pub(crate) handler: &'a mut dyn InputHandler,
}

impl KeyRouter<'_> {
    /// Returns whether an element or a shortcut used the key.
    pub(crate) fn route(self, innermost: ElementId, event: &KeyEvent) -> bool {
        let mut receiver = Some(innermost);
        while let Some(element) = receiver {
            if self.handler.key_event(element, event) == Propagation::Stop {
                return true;
            }
            if let Some(shortcut) = self.shortcut_in(ShortcutScope::Element(element), event) {
                self.handler.shortcut(shortcut);
                return true;
            }
            receiver = self.elements[element].parent;
        }
        if let Some(shortcut) = self.shortcut_in(ShortcutScope::Surface, event) {
            self.handler.shortcut(shortcut);
            return true;
        }
        false
    }

    /// The shortcut of the scope that the press asks for; of two with one chord, the one
    /// registered later.
    fn shortcut_in(&self, scope: ShortcutScope, event: &KeyEvent) -> Option<ShortcutId> {
        if !event.is_press() {
            return None;
        }
        let chord = event.chord();
        self.shortcuts
            .iter()
            .rev()
            .find(|shortcut| shortcut.scope == scope && shortcut.chord == chord)
            .map(|shortcut| shortcut.id)
    }
}
