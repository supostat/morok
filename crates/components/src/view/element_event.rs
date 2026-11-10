use morok_core::{AccessibilityAction, KeyEvent, PointerEvent};

/// What reached an element of a surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ElementEvent<'a> {
    /// The pointer did something to the element.
    Pointer(PointerEvent),
    /// A key went down or up while the element, or an element inside it, had focus.
    Key(KeyEvent),
    /// An assistive technology asks the element for one of the actions it offers.
    Accessibility(AccessibilityAction<'a>),
}
