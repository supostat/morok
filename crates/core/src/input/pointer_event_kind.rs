use crate::input::pointer_button::PointerButton;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerEventKind {
    /// The pointer came over the element. Told to every element it came over, the outermost
    /// first, whatever they answer.
    Entered,
    /// The pointer is no longer over the element. Told to every element it left, the innermost
    /// first, whatever they answer.
    Left,
    Moved,
    Pressed {
        button: PointerButton,
        /// How many presses in quick succession this one completes: two for a double click.
        click_count: u32,
    },
    Released {
        button: PointerButton,
    },
    /// The primary button was pressed and released over the element: over the innermost
    /// element around both the place of the press and the place of the release.
    Clicked {
        click_count: u32,
    },
}
