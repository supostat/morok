use cssparser::{Parser, Token};
use morok_style::Pixels;

use super::{Parsed, rejected};

/// A length in px; a unitless zero is a length too, as in CSS.
pub(crate) fn parse(input: &mut Parser<'_>) -> Parsed<Pixels> {
    match input.next()?.clone() {
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("px") && value.is_finite() => {
            Ok(Pixels(value))
        }
        Token::Number { value: 0.0, .. } => Ok(Pixels(0.0)),
        _ => rejected(),
    }
}
