use crate::leading::Leading;
use crate::token::Token;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineHeight {
    /// The line height the font asks for itself.
    Normal,
    Token(Token<Leading>),
}
