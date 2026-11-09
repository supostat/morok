use crate::color::Color;
use crate::token::Token;

/// A color in a declaration: a color token or one of the two keywords the subset admits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorValue {
    Token(Token<Color>),
    Transparent,
    CurrentColor,
}
