use crate::shadow::Shadow;
use crate::token::Token;

/// The shadows of a box: none, or the layers a shadow token holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoxShadow {
    None,
    Token(Token<Shadow>),
}
