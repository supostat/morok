use crate::input::key::Key;
use crate::input::modifiers::Modifiers;

/// A key with the modifiers held as it goes down: what a shortcut answers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub key: Key,
    pub modifiers: Modifiers,
}
