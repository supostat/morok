use crate::pixels::Pixels;
use crate::token::Token;

/// A length token multiplied by a number, as a term of a length expression.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TokenTerm {
    pub token: Token<Pixels>,
    pub factor: f32,
}
