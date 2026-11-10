use crate::easing::Easing;
use crate::token::Token;

/// An easing in a declaration: a curve written out or a `motion-` token.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EasingValue {
    Curve(Easing),
    Token(Token<Easing>),
}
