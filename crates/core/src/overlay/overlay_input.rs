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
    pub const fn takes_pointer(self) -> bool {
        !matches!(self, Self::None)
    }

    pub const fn takes_keyboard(self) -> bool {
        matches!(self, Self::PointerAndKeyboard)
    }
}
