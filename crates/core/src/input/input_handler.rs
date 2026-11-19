use crate::accessibility::accessibility_action::AccessibilityAction;
use crate::drag::drag_event::DragEvent;
use crate::drag::drag_outcome::DragOutcome;
use crate::drag::drag_start::DragStart;
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

    /// The user changed the text the element edits: `text` is what it holds now.
    fn text_changed(&mut self, element: ElementId, text: &str);

    /// The pointer began to drag the element, a drag source. Returns what the drag carries
    /// and how far it goes; none leaves the press a press.
    fn drag_started(&mut self, source: ElementId) -> Option<DragStart>;

    /// A drag did something to the element, a drop target that takes a type the drag carries.
    fn drag_event(&mut self, target: ElementId, event: &DragEvent<'_>);

    /// The drag that began on the element, a drag source, is over.
    fn drag_ended(&mut self, source: ElementId, outcome: DragOutcome);
}
