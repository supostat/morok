use crate::input::focus_direction::FocusDirection;
use crate::input::key::Key;
use crate::input::key_chord::KeyChord;
use crate::input::key_phase::KeyPhase;
use crate::input::modifiers::Modifiers;

/// A key that went down or up while the surface had the keyboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub phase: KeyPhase,
    pub key: Key,
    pub modifiers: Modifiers,
    /// Whether the key is held down and the system repeats it.
    pub is_repeat: bool,
}

impl KeyEvent {
    pub fn chord(&self) -> KeyChord {
        KeyChord {
            key: self.key,
            modifiers: self.modifiers,
        }
    }

    pub(crate) fn is_press(&self) -> bool {
        self.phase == KeyPhase::Pressed
    }

    /// Whether the press tells that the user works with the keyboard. A command such as
    /// Command-C tells nothing: it is given with the pointer in hand as well.
    pub(crate) fn shows_keyboard_use(&self) -> bool {
        self.is_press() && !self.modifiers.intersects(Modifiers::COMMAND.union(Modifiers::CONTROL))
    }

    /// Where the press moves focus when no element and no shortcut uses it.
    pub(crate) fn focus_move(&self) -> Option<FocusDirection> {
        if !self.is_press() || self.key != Key::Tab {
            return None;
        }
        if self.modifiers.is_empty() {
            Some(FocusDirection::Forward)
        } else if self.modifiers == Modifiers::SHIFT {
            Some(FocusDirection::Backward)
        } else {
            None
        }
    }
}
