use cssparser::{Parser, Token};
use morok_style::{Length, LengthPercentage, LengthPercentageAuto};

use crate::compiled_tokens::CompiledTokens;
use crate::linear::Linear;
use crate::problem::Problem;
use crate::property_syntax::{accepts_keyword, calc};
use crate::span::Span;
use crate::token_reference;
use crate::value_error::{ValueError, ValueResult};

const PIXELS: &str = "px";

pub(crate) fn unsupported_unit(unit: &str, span: Span) -> ValueError {
    Problem {
        span,
        message: format!("the unit `{unit}` is not supported"),
        hint: "lengths are in px or percent".to_owned(),
    }
    .into()
}

/// One length or percentage: a literal, a length token or a `calc()`.
fn linear(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Linear> {
    if token_reference::is_ahead(input) {
        return Ok(Linear::token(token_reference::parse(tokens, input)?.length()?));
    }
    if calc::is_ahead(input) {
        return calc::parse(tokens, input);
    }
    input.skip_whitespace();
    let start = input.position();
    match input.next().map_err(|_| ValueError::Mismatch)?.clone() {
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case(PIXELS) => {
            if value.is_finite() {
                Ok(Linear::pixels(value))
            } else {
                Err(ValueError::Mismatch)
            }
        }
        Token::Dimension { unit, .. } => Err(unsupported_unit(&unit, Span::between(start, input.position()))),
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => Ok(Linear::fraction(unit_value)),
        Token::Number { value: 0.0, .. } => Ok(Linear::default()),
        _ => Err(ValueError::Mismatch),
    }
}

fn non_negative(linear: Linear) -> ValueResult<Linear> {
    if linear.is_negative_literal() {
        Err(ValueError::Mismatch)
    } else {
        Ok(linear)
    }
}

fn without_percentage(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Linear> {
    input.skip_whitespace();
    let start = input.position();
    let linear = linear(tokens, input)?;
    if linear.has_percentage() {
        return Err(Problem {
            span: Span::between(start, input.position()),
            message: "a percentage is not supported here".to_owned(),
            hint: "use a length in px or a length token".to_owned(),
        }
        .into());
    }
    Ok(linear)
}

pub(crate) fn length_percentage(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<LengthPercentage> {
    Ok(linear(tokens, input)?.into_length_percentage())
}

pub(crate) fn non_negative_length_percentage(
    tokens: &CompiledTokens,
    input: &mut Parser<'_>,
) -> ValueResult<LengthPercentage> {
    Ok(non_negative(linear(tokens, input)?)?.into_length_percentage())
}

pub(crate) fn length(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Length> {
    Ok(without_percentage(tokens, input)?.into_length())
}

pub(crate) fn non_negative_length(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Length> {
    Ok(non_negative(without_percentage(tokens, input)?)?.into_length())
}

/// A length or percentage, or the keyword that stands for the automatic value of the property.
pub(crate) fn length_percentage_or(
    automatic_keyword: &str,
    tokens: &CompiledTokens,
    input: &mut Parser<'_>,
) -> ValueResult<LengthPercentageAuto> {
    if accepts_keyword(input, automatic_keyword) {
        return Ok(LengthPercentageAuto::Auto);
    }
    Ok(LengthPercentageAuto::LengthPercentage(length_percentage(
        tokens, input,
    )?))
}

pub(crate) fn non_negative_length_percentage_or(
    automatic_keyword: &str,
    tokens: &CompiledTokens,
    input: &mut Parser<'_>,
) -> ValueResult<LengthPercentageAuto> {
    if accepts_keyword(input, automatic_keyword) {
        return Ok(LengthPercentageAuto::Auto);
    }
    Ok(LengthPercentageAuto::LengthPercentage(non_negative_length_percentage(
        tokens, input,
    )?))
}
