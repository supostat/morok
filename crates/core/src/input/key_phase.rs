/// Whether a key went down or came up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyPhase {
    Pressed,
    Released,
}
