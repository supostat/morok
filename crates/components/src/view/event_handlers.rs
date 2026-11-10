use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Result};

use morok_core::{ElementId, ElementTree, Propagation, ShortcutId};
use morok_reactive::Runtime;

use crate::view::element_event::ElementEvent;
use crate::view::shortcut_handler::ShortcutHandler;
use crate::view::surfaces::Surfaces;

type ElementHandler = Box<dyn FnMut(&mut Runtime<Surfaces>, &ElementEvent<'_>) -> Propagation>;

/// What the components of a surface do with the input the tree routes to their elements. A
/// handler writes signals; the effects of the next flush change the tree.
#[derive(Default)]
pub(crate) struct EventHandlers {
    of_elements: HashMap<ElementId, ElementHandler>,
    of_shortcuts: HashMap<ShortcutId, ShortcutHandler>,
}

impl EventHandlers {
    /// Names what hears of the events of the element, in place of what heard of them before.
    pub(crate) fn set_element_handler(&mut self, element: ElementId, handler: ElementHandler) {
        self.of_elements.insert(element, handler);
    }

    pub(crate) fn set_shortcut_handler(&mut self, shortcut: ShortcutId, handler: ShortcutHandler) {
        self.of_shortcuts.insert(shortcut, handler);
    }

    /// Tells the handler of the element of the event. An element without one does not use it.
    pub(crate) fn element_event(
        &mut self,
        runtime: &mut Runtime<Surfaces>,
        element: ElementId,
        event: &ElementEvent<'_>,
    ) -> Propagation {
        self.of_elements
            .get_mut(&element)
            .map_or(Propagation::Continue, |handler| handler(runtime, event))
    }

    pub(crate) fn shortcut(&mut self, runtime: &mut Runtime<Surfaces>, shortcut: ShortcutId) {
        let handler = self
            .of_shortcuts
            .get_mut(&shortcut)
            .expect("a shortcut of a surface is registered with its handler");
        (handler.run)(runtime);
    }

    /// The element is removed: nothing hears of its events from now on.
    pub(crate) fn forget_element(&mut self, element: ElementId) {
        self.of_elements.remove(&element);
    }

    /// Drops the handlers of the shortcuts whose elements are no longer in the tree: the tree
    /// dropped those shortcuts with the elements.
    pub(crate) fn forget_shortcuts_of_removed_elements(&mut self, tree: &ElementTree) {
        self.of_shortcuts.retain(|_, handler| tree.contains(handler.element));
    }
}

impl Debug for EventHandlers {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("EventHandlers")
            .field("elements", &self.of_elements.len())
            .field("shortcuts", &self.of_shortcuts.len())
            .finish()
    }
}
