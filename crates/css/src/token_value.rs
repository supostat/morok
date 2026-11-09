use std::mem;

use cssparser::Parser;
use morok_style::{Color, Easing, FontFamilies, FontWeight, Leading, Milliseconds, Pixels, Shadow};

use crate::token_kind::TokenKind;
use crate::value_syntax::{Parsed, color, font_families, length, motion, number, shadow};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TokenValue {
    Color(Color),
    Length(Pixels),
    FontFamilies(FontFamilies),
    Leading(Leading),
    FontWeight(FontWeight),
    Shadow(Shadow),
    Duration(Milliseconds),
    Easing(Easing),
}

impl TokenValue {
    pub(crate) fn parse(kind: TokenKind, input: &mut Parser<'_>) -> Parsed<Self> {
        match kind {
            TokenKind::Color => color::parse(input).map(Self::Color),
            TokenKind::Length => length::parse(input).map(Self::Length),
            TokenKind::FontFamilies => font_families::parse(input).map(Self::FontFamilies),
            TokenKind::Leading => number::leading(input).map(Self::Leading),
            TokenKind::FontWeight => number::font_weight(input).map(Self::FontWeight),
            TokenKind::Shadow => shadow::parse(input).map(Self::Shadow),
            TokenKind::Motion => match input.try_parse(motion::duration) {
                Ok(duration) => Ok(Self::Duration(duration)),
                Err(_) => motion::easing(input).map(Self::Easing),
            },
        }
    }

    pub(crate) fn is_same_kind_as(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }

    pub(crate) fn description(&self) -> &'static str {
        match self {
            Self::Color(_) => "a color",
            Self::Length(_) => "a length",
            Self::FontFamilies(_) => "a list of font families",
            Self::Leading(_) => "a line height",
            Self::FontWeight(_) => "a font weight",
            Self::Shadow(_) => "a shadow",
            Self::Duration(_) => "a duration",
            Self::Easing(_) => "an easing",
        }
    }
}
