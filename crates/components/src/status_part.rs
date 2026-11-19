use std::fmt::{Debug, Formatter, Result};

use morok_core::KeyChord;

use crate::view::{Action, Property};

/// What stands in a status bar, in its order.
pub(crate) enum StatusPart {
    /// A line of text that follows its source.
    Text(Property<String>),
    /// A command with the chord that gives it, which a press of the part gives as well.
    Command {
        chord: KeyChord,
        label: String,
        on_press: Action,
    },
    /// Room that takes what the other parts leave and pushes what follows to the far end.
    Spacer,
}

impl Debug for StatusPart {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self {
            Self::Text(text) => formatter.debug_tuple("Text").field(text).finish(),
            Self::Command { chord, label, .. } => formatter
                .debug_struct("Command")
                .field("chord", chord)
                .field("label", label)
                .finish_non_exhaustive(),
            Self::Spacer => formatter.write_str("Spacer"),
        }
    }
}
