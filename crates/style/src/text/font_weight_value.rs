use crate::font_weight::FontWeight;
use crate::token::Token;

/// A `font-weight`: a weight token, or the weight a CSS keyword stands for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontWeightValue {
    Token(Token<FontWeight>),
    Keyword(FontWeight),
}
