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
/// shortcuts of the surface come last. Between the offer to the focused element and its
/// shortcuts the tree lets the text that element edits take the key.
pub(crate) struct KeyRouter<'a> {
    pub(crate) elements: &'a ElementArena,
    pub(crate) shortcuts: &'a [Shortcut],
    pub(crate) handler: &'a mut dyn InputHandler,
}

impl KeyRouter<'_> {
    /// Offers the key to the element alone. Returns whether the element used it.
    pub(crate) fn offer(&mut self, element: ElementId, event: &KeyEvent) -> bool {
        self.handler.key_event(element, event) == Propagation::Stop
    }

    /// Goes on from an element that was offered the key and did not use it: its shortcuts,
    /// then each element around it with its shortcuts, then the shortcuts of the surface.
    /// Returns whether an element or a shortcut used the key.
    pub(crate) fn route_past(mut self, offered: ElementId, event: &KeyEvent) -> bool {
        let mut receiver = Some(offered);
        while let Some(element) = receiver {
            if element != offered && self.offer(element, event) {
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
