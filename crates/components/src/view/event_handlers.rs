use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Result};

use morok_core::{DragStart, ElementId, ElementTree, Propagation, ShortcutId};
use morok_reactive::Runtime;

use crate::view::element_event::ElementEvent;
use crate::view::shortcut_handler::ShortcutHandler;
use crate::view::surfaces::Surfaces;

type ElementHandler = Box<dyn FnMut(&mut Runtime<Surfaces>, &ElementEvent<'_>) -> Propagation>;
type DragSourceHandler = Box<dyn FnMut(&mut Runtime<Surfaces>) -> Option<DragStart>>;

/// What the components of a surface do with the input the tree routes to their elements. A
/// handler writes signals; the effects of the next flush change the tree.
#[derive(Default)]
pub(crate) struct EventHandlers {
    elements: HashMap<ElementId, ElementHandler>,
    shortcuts: HashMap<ShortcutId, ShortcutHandler>,
    drag_sources: HashMap<ElementId, DragSourceHandler>,
}

impl EventHandlers {
    /// Names what hears of the events of the element, in place of what heard of them before.
    pub(crate) fn set_element_handler(&mut self, element: ElementId, handler: ElementHandler) {
        self.elements.insert(element, handler);
    }

    pub(crate) fn set_shortcut_handler(&mut self, shortcut: ShortcutId, handler: ShortcutHandler) {
        self.shortcuts.insert(shortcut, handler);
    }

    /// Names what says what a drag that begins on the element carries, in place of what said
    /// it before.
    pub(crate) fn set_drag_source_handler(&mut self, source: ElementId, handler: DragSourceHandler) {
        self.drag_sources.insert(source, handler);
    }

    /// Asks the handler of the drag source what the drag that begins on it carries. A source
    /// without one starts no drag.
    pub(crate) fn drag_started(&mut self, runtime: &mut Runtime<Surfaces>, source: ElementId) -> Option<DragStart> {
        self.drag_sources.get_mut(&source)?(runtime)
    }

    /// Tells the handler of the element of the event. An element without one does not use it.
    pub(crate) fn element_event(
        &mut self,
        runtime: &mut Runtime<Surfaces>,
        element: ElementId,
        event: &ElementEvent<'_>,
    ) -> Propagation {
        self.elements
            .get_mut(&element)
            .map_or(Propagation::Continue, |handler| handler(runtime, event))
    }

    pub(crate) fn shortcut(&mut self, runtime: &mut Runtime<Surfaces>, shortcut: ShortcutId) {
        let handler = self
            .shortcuts
            .get_mut(&shortcut)
            .expect("a shortcut of a surface is registered with its handler");
        (handler.run)(runtime);
    }

    /// The element is removed: nothing hears of its events from now on.
    pub(crate) fn forget_element(&mut self, element: ElementId) {
        self.elements.remove(&element);
        self.drag_sources.remove(&element);
    }

    /// Drops the handlers of the shortcuts whose elements are no longer in the tree: the tree
    /// dropped those shortcuts with the elements.
    pub(crate) fn forget_shortcuts_of_removed_elements(&mut self, tree: &ElementTree) {
        self.shortcuts.retain(|_, handler| tree.contains(handler.element));
    }
}

impl Debug for EventHandlers {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("EventHandlers")
            .field("elements", &self.elements.len())
            .field("shortcuts", &self.shortcuts.len())
            .field("drag_sources", &self.drag_sources.len())
            .finish()
    }
}
