use std::borrow::Cow;

use crate::pixels::Pixels;
use crate::token_term::TokenTerm;

/// A length that may have a part relative to a percentage basis: `pixels + fraction × basis + Σ factor × token`.
///
/// Every CSS `<length-percentage>` reduces to this form: `50%` has a fraction of 0.5 and
/// `calc(100% - var(--space-4))` a fraction of 1.0 and one token term with a factor of -1.0.
/// A value written without a percentage has no fraction; `0%` has a fraction of zero, and the two
/// differ, as in CSS, when the basis is indefinite.
#[derive(Clone, Debug, PartialEq)]
pub struct LengthPercentage {
    pub pixels: Pixels,
    pub fraction: Option<f32>,
    pub tokens: Cow<'static, [TokenTerm]>,
}
