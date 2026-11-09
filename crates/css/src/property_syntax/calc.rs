use cssparser::{Parser, Token};

use crate::compiled_tokens::CompiledTokens;
use crate::linear::Linear;
use crate::property_syntax::length::unsupported_unit;
use crate::property_syntax::{nested, peek};
use crate::span::Span;
use crate::token_reference;
use crate::value_error::{ValueError, ValueResult};

const CALC: &str = "calc";
const PIXELS: &str = "px";

/// A `calc()` operand: a plain number or a length expression.
enum Operand {
    Number(f32),
    Length(Linear),
}

pub(crate) fn is_ahead(input: &mut Parser<'_>) -> bool {
    matches!(peek(input), Some(Token::Function(name)) if name.eq_ignore_ascii_case(CALC))
}

/// Reads `calc(…)` and folds it; the expression must be a length, not a plain number.
pub(crate) fn parse(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Linear> {
    if !matches!(input.next(), Ok(Token::Function(name)) if name.eq_ignore_ascii_case(CALC)) {
        return Err(ValueError::Mismatch);
    }
    match nested(input, |expression| sum(tokens, expression))? {
        Operand::Length(length) if length.is_finite() => Ok(length),
        _ => Err(ValueError::Mismatch),
    }
}

fn sum(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Operand> {
    let mut total = product(tokens, input)?;
    loop {
        let sign = match input.next() {
            Ok(Token::Delim('+')) => 1.0,
            Ok(Token::Delim('-')) => -1.0,
            Ok(_) => return Err(ValueError::Mismatch),
            Err(_) => return Ok(total),
        };
        total = match (total, product(tokens, input)?) {
            (Operand::Number(left), Operand::Number(right)) => Operand::Number(left + sign * right),
            (Operand::Length(left), Operand::Length(right)) => Operand::Length(left.plus(right.scaled(sign))),
            _ => return Err(ValueError::Mismatch),
        };
    }
}

fn product(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Operand> {
    let mut result = operand(tokens, input)?;
    loop {
        match peek(input) {
            Some(Token::Delim('*')) => {
                input.next().map_err(|_| ValueError::Mismatch)?;
                result = match (result, operand(tokens, input)?) {
                    (Operand::Number(left), Operand::Number(right)) => Operand::Number(left * right),
                    (Operand::Number(factor), Operand::Length(length))
                    | (Operand::Length(length), Operand::Number(factor)) => Operand::Length(length.scaled(factor)),
                    (Operand::Length(_), Operand::Length(_)) => return Err(ValueError::Mismatch),
                };
            }
            Some(Token::Delim('/')) => {
                input.next().map_err(|_| ValueError::Mismatch)?;
                let Operand::Number(divisor) = operand(tokens, input)? else {
                    return Err(ValueError::Mismatch);
                };
                if divisor == 0.0 {
                    return Err(ValueError::Mismatch);
                }
                result = match result {
                    Operand::Number(dividend) => Operand::Number(dividend / divisor),
                    Operand::Length(length) => Operand::Length(length.scaled(1.0 / divisor)),
                };
            }
            _ => return Ok(result),
        }
    }
}

fn operand(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Operand> {
    if token_reference::is_ahead(input) {
        let token = token_reference::parse(tokens, input)?.length()?;
        return Ok(Operand::Length(Linear::token(token)));
    }
    input.skip_whitespace();
    let start = input.position();
    match input.next().map_err(|_| ValueError::Mismatch)?.clone() {
        Token::Number { value, .. } => Ok(Operand::Number(value)),
        Token::Percentage { unit_value, .. } => Ok(Operand::Length(Linear::fraction(unit_value))),
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case(PIXELS) => {
            Ok(Operand::Length(Linear::pixels(value)))
        }
        Token::Dimension { unit, .. } => Err(unsupported_unit(&unit, Span::between(start, input.position()))),
        Token::ParenthesisBlock => nested(input, |group| sum(tokens, group)),
        Token::Function(name) if name.eq_ignore_ascii_case(CALC) => nested(input, |group| sum(tokens, group)),
        _ => Err(ValueError::Mismatch),
    }
}
