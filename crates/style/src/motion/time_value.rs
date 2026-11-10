use crate::milliseconds::Milliseconds;
use crate::token::Token;

/// A time in a declaration: milliseconds written out or a `motion-` token. A delay may be
/// negative; a duration may not.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimeValue {
    Milliseconds(Milliseconds),
    Token(Token<Milliseconds>),
}
