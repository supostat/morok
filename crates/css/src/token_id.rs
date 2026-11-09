use morok_style::{Color, Easing, FontFamilies, FontWeight, Leading, Milliseconds, Pixels, Shadow, Token};

use crate::value_kind::ValueKind;

/// The typed ID of a compiled token.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenId {
    Color(Token<Color>),
    Length(Token<Pixels>),
    FontFamilies(Token<FontFamilies>),
    Leading(Token<Leading>),
    FontWeight(Token<FontWeight>),
    Shadow(Token<Shadow>),
    Duration(Token<Milliseconds>),
    Easing(Token<Easing>),
}

impl TokenId {
    pub(crate) fn kind(self) -> ValueKind {
        match self {
            Self::Color(_) => ValueKind::Color,
            Self::Length(_) => ValueKind::Length,
            Self::FontFamilies(_) => ValueKind::FontFamilies,
            Self::Leading(_) => ValueKind::Leading,
            Self::FontWeight(_) => ValueKind::FontWeight,
            Self::Shadow(_) => ValueKind::Shadow,
            Self::Duration(_) => ValueKind::Duration,
            Self::Easing(_) => ValueKind::Easing,
        }
    }

    pub(crate) fn description(self) -> &'static str {
        self.kind().description()
    }

    pub(crate) fn index(self) -> usize {
        match self {
            Self::Color(token) => token.index(),
            Self::Length(token) => token.index(),
            Self::FontFamilies(token) => token.index(),
            Self::Leading(token) => token.index(),
            Self::FontWeight(token) => token.index(),
            Self::Shadow(token) => token.index(),
            Self::Duration(token) => token.index(),
            Self::Easing(token) => token.index(),
        }
    }
}
