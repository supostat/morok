/// What the user last worked with. Focus is shown only to a user of the keyboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InputModality {
    Keyboard,
    Pointer,
}
