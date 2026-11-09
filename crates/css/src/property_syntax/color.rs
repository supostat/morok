use cssparser::{Parser, Token};
use morok_style::ColorValue;

use crate::compiled_tokens::CompiledTokens;
use crate::problem::Problem;
use crate::property_syntax::{nested, peek};
use crate::span::Span;
use crate::token_id::TokenId;
use crate::token_reference;
use crate::value_error::{ValueError, ValueResult};

const TRANSPARENT: &str = "transparent";
const CURRENT_COLOR: &str = "currentcolor";
const RAW_COLOR_FUNCTIONS: [&str; 10] = [
    "rgb", "rgba", "hsl", "hsla", "hwb", "lab", "lch", "oklab", "oklch", "color",
];

fn is_raw_color_function(name: &str) -> bool {
    RAW_COLOR_FUNCTIONS
        .iter()
        .any(|function| name.eq_ignore_ascii_case(function))
}

/// Whether a color comes next. A raw color counts, so that reading it reports it.
pub(crate) fn is_ahead(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<bool> {
    if let Some(id) = token_reference::id_ahead(tokens, input)? {
        return Ok(matches!(id, TokenId::Color(_)));
    }
    Ok(match peek(input) {
        Some(Token::Ident(name)) => name.eq_ignore_ascii_case(TRANSPARENT) || name.eq_ignore_ascii_case(CURRENT_COLOR),
        Some(Token::Hash(_) | Token::IDHash(_)) => true,
        Some(Token::Function(name)) => is_raw_color_function(&name),
        _ => false,
    })
}

pub(crate) fn parse(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<ColorValue> {
    if token_reference::is_ahead(input) {
        return Ok(ColorValue::Token(token_reference::parse(tokens, input)?.color()?));
    }
    input.skip_whitespace();
    let start = input.position();
    match input.next().map_err(|_| ValueError::Mismatch)?.clone() {
        Token::Ident(name) if name.eq_ignore_ascii_case(TRANSPARENT) => Ok(ColorValue::Transparent),
        Token::Ident(name) if name.eq_ignore_ascii_case(CURRENT_COLOR) => Ok(ColorValue::CurrentColor),
        Token::Hash(_) | Token::IDHash(_) => Err(raw_color(Span::between(start, input.position()))),
        Token::Function(name) if is_raw_color_function(&name) => {
            nested(input, |arguments| {
                while arguments.next().is_ok() {}
                Ok(())
            })?;
            Err(raw_color(Span::between(start, input.position())))
        }
        _ => Err(ValueError::Mismatch),
    }
}

fn raw_color(span: Span) -> ValueError {
    Problem {
        span,
        message: "raw colors are not supported".to_owned(),
        hint: "a color comes from a `color-` token: `var(--color-…)`".to_owned(),
    }
    .into()
}
