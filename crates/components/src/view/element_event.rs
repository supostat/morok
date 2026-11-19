use morok_core::{AccessibilityAction, DragEvent, DragOutcome, KeyEvent, PointerEvent};

/// What reached an element of a surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ElementEvent<'a> {
    /// The pointer did something to the element.
    Pointer(PointerEvent),
    /// A key went down or up while the element, or an element inside it, had focus.
    Key(KeyEvent),
    /// An assistive technology asks the element for one of the actions it offers.
    Accessibility(AccessibilityAction<'a>),
    /// The user changed the text the element edits; this is what it holds now.
    TextChanged(&'a str),
    /// A drag did something to the element, a drop target that takes a type the drag carries.
    Drag(DragEvent<'a>),
    /// The drag that began on the element, a drag source, is over.
    DragEnded(DragOutcome),
}
