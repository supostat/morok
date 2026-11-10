use morok_core::{AccessibilityAction, ElementId, InputHandler, KeyEvent, PointerEvent, Propagation, ShortcutId};
use morok_reactive::Runtime;

use crate::view::element_event::ElementEvent;
use crate::view::event_handlers::EventHandlers;
use crate::view::surfaces::Surfaces;

/// The input handler of the tree of a surface: hands what the tree routes to an element to
/// the handler a component gave the element.
pub(crate) struct EventDispatch<'a> {
    pub(crate) handlers: &'a mut EventHandlers,
    pub(crate) runtime: &'a mut Runtime<Surfaces>,
}

impl InputHandler for EventDispatch<'_> {
    fn pointer_event(&mut self, element: ElementId, event: &PointerEvent) -> Propagation {
        self.handlers
            .element_event(self.runtime, element, &ElementEvent::Pointer(*event))
    }

    fn key_event(&mut self, element: ElementId, event: &KeyEvent) -> Propagation {
        self.handlers
            .element_event(self.runtime, element, &ElementEvent::Key(*event))
    }

    fn shortcut(&mut self, shortcut: ShortcutId) {
        self.handlers.shortcut(self.runtime, shortcut);
    }

    fn accessibility_action(&mut self, element: ElementId, action: &AccessibilityAction<'_>) {
        self.handlers
            .element_event(self.runtime, element, &ElementEvent::Accessibility(*action));
    }

    fn text_changed(&mut self, element: ElementId, text: &str) {
        self.handlers
            .element_event(self.runtime, element, &ElementEvent::TextChanged(text));
    }
}
