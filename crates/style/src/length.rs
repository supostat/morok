use std::borrow::Cow;

use crate::pixels::Pixels;
use crate::token_term::TokenTerm;

/// A length that does not depend on a percentage basis: `pixels + Σ factor × token`.
#[derive(Clone, Debug, PartialEq)]
pub struct Length {
    pub pixels: Pixels,
    pub tokens: Cow<'static, [TokenTerm]>,
}
