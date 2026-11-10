use crate::input::pointer_button::PointerButton;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerInputKind {
    Moved,
    Pressed {
        button: PointerButton,
        /// How many presses in quick succession this one completes: two for a double click.
        click_count: u32,
    },
    Released {
        button: PointerButton,
        click_count: u32,
    },
    /// The pointer left the surface.
    Exited,
}
