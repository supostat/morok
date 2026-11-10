use crate::accessibility::accessibility_action::AccessibilityAction;
use crate::element_id::ElementId;
use crate::input::key_event::KeyEvent;
use crate::input::pointer_event::PointerEvent;
use crate::input::propagation::Propagation;
use crate::input::shortcut_id::ShortcutId;

/// The port through which the elements of a tree hear of the input routed to them. The tree
/// decides which element an event belongs to; what the element does with it is the business of
/// whoever built the tree.
pub trait InputHandler {
    /// The pointer did something to the element. An event the element does not use goes on to
    /// the element around it, up to the root.
    fn pointer_event(&mut self, element: ElementId, event: &PointerEvent) -> Propagation;

    /// A key went down or up while the element, or an element inside it, had focus. With no
    /// focus the root is asked alone.
    fn key_event(&mut self, element: ElementId, event: &KeyEvent) -> Propagation;

    /// The chord of a registered shortcut was pressed and no element between focus and the
    /// scope of the shortcut used the key.
    fn shortcut(&mut self, shortcut: ShortcutId);

    /// An assistive technology asks the element for one of the actions the element offers.
    fn accessibility_action(&mut self, element: ElementId, action: &AccessibilityAction<'_>);
}
