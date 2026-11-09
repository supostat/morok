use cssparser::{Parser, Token};
use morok_style::{FontWeight, Leading};

use super::{Parsed, rejected};

const FONT_WEIGHTS: std::ops::RangeInclusive<i32> = 1..=1000;

pub(crate) fn finite(input: &mut Parser<'_>) -> Parsed<f32> {
    let value = input.expect_number()?;
    if value.is_finite() { Ok(value) } else { rejected() }
}

pub(crate) fn unit_interval(input: &mut Parser<'_>) -> Parsed<f32> {
    let value = input.expect_number()?;
    if (0.0..=1.0).contains(&value) {
        Ok(value)
    } else {
        rejected()
    }
}

pub(crate) fn leading(input: &mut Parser<'_>) -> Parsed<Leading> {
    match input.next()?.clone() {
        Token::Number { value, .. } if value > 0.0 && value.is_finite() => Ok(Leading(value)),
        _ => rejected(),
    }
}

pub(crate) fn font_weight(input: &mut Parser<'_>) -> Parsed<FontWeight> {
    match input.next()?.clone() {
        Token::Number {
            int_value: Some(weight),
            ..
        } if FONT_WEIGHTS.contains(&weight) => Ok(FontWeight(weight as u16)),
        _ => rejected(),
    }
}
