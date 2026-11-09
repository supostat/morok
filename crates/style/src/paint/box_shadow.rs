use crate::shadow::Shadow;
use crate::token::Token;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoxShadow {
    None,
    Token(Token<Shadow>),
}
