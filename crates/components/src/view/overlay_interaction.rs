use std::fmt::{Debug, Formatter, Result};

use morok_core::OverlayInput;

use crate::view::action::Action;

/// What an overlay does with the pointer and the keyboard.
pub enum OverlayInteraction {
    /// It only shows: the pointer and the keys pass it by.
    Passive,
    /// It takes the pointer, and the keyboard stays with the surface it is opened over, as
    /// for a list that drops down under the control the user types into. `on_dismiss` asks it
    /// to go: on a press outside it and when the user turns away from the application.
    PointerOnly { on_dismiss: Action },
    /// It takes the pointer and the keyboard, and `on_dismiss` asks it to go: on a press
    /// outside it, on Escape that nothing inside it used, and when the user turns away from
    /// the application.
    Interactive { on_dismiss: Action },
}

impl OverlayInteraction {
    /// What of the input of the user the overlay takes.
    pub(crate) const fn input(&self) -> OverlayInput {
        match self {
            Self::Passive => OverlayInput::None,
            Self::PointerOnly { .. } => OverlayInput::Pointer,
            Self::Interactive { .. } => OverlayInput::PointerAndKeyboard,
        }
    }

    /// What asks the overlay to go; none for an overlay that only shows.
    pub(crate) fn on_dismiss(&mut self) -> Option<&mut Action> {
        match self {
            Self::Passive => None,
            Self::PointerOnly { on_dismiss } | Self::Interactive { on_dismiss } => Some(on_dismiss),
        }
    }
}

impl Debug for OverlayInteraction {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter.write_str(match self {
            Self::Passive => "Passive",
            Self::PointerOnly { .. } => "PointerOnly",
            Self::Interactive { .. } => "Interactive",
        })
    }
}
