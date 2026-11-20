use morok::style::Theme;

use crate::{ember, tokens};

/// A set of values for the tokens of the catalog: the windows show one at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenSet {
    /// The values of the base theme of the kit.
    Base,
    /// A warm accent and warm neutrals.
    Ember,
}

impl TokenSet {
    pub fn title(self) -> &'static str {
        match self {
            Self::Base => "Base",
            Self::Ember => "Ember",
        }
    }

    /// The values of the tokens under the light color scheme.
    pub fn light(self) -> Theme {
        match self {
            Self::Base => tokens::LIGHT,
            Self::Ember => ember::LIGHT,
        }
    }

    /// The values of the tokens under the dark color scheme.
    pub fn dark(self) -> Theme {
        match self {
            Self::Base => tokens::DARK,
            Self::Ember => ember::DARK,
        }
    }
}
