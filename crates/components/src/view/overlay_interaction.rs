use std::fmt::{Debug, Formatter, Result};

use crate::view::action::Action;

/// What an overlay does with the pointer and the keyboard.
pub enum OverlayInteraction {
    /// It only shows: the pointer and the keys pass it by.
    Passive,
    /// It takes the pointer and the keyboard, and `on_dismiss` asks it to go: on a press
    /// outside it, on Escape that nothing inside it used, and when the user turns away from
    /// the application.
    Interactive { on_dismiss: Action },
}

impl Debug for OverlayInteraction {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter.write_str(match self {
            Self::Passive => "Passive",
            Self::Interactive { .. } => "Interactive",
        })
    }
}
