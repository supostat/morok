/// What of the input of the user an overlay takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OverlayInput {
    /// Nothing: the pointer and the keys pass it by to what lies under it.
    None,
    /// The pointer alone: the keyboard stays with the surface the overlay is opened over.
    Pointer,
    /// The pointer and the keyboard.
    PointerAndKeyboard,
}

impl OverlayInput {
    /// Whether the pointer acts on the overlay and not on what lies under it.
    pub const fn takes_pointer(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Whether the keys go to the overlay and not to the surface it is opened over.
    pub const fn takes_keyboard(self) -> bool {
        matches!(self, Self::PointerAndKeyboard)
    }
}
