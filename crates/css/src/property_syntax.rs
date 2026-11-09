pub(crate) mod background;
pub(crate) mod border;
pub(crate) mod calc;
pub(crate) mod color;
pub(crate) mod flex;
pub(crate) mod gradient;
pub(crate) mod grid;
pub(crate) mod layout;
pub(crate) mod length;
pub(crate) mod text;
pub(crate) mod track;

use cssparser::{ParseError, ParseErrorKind, Parser, Token};

use crate::value_error::{ValueError, ValueResult};

/// The next token, without consuming it.
pub(crate) fn peek<'i>(input: &mut Parser<'i>) -> Option<Token<'i>> {
    let before = input.state();
    let token = input.next().ok().cloned();
    input.reset(&before);
    token
}

/// Reads the block or function arguments just opened; the reader must consume all of them.
pub(crate) fn nested<'i, T>(
    input: &mut Parser<'i>,
    read: impl FnOnce(&mut Parser<'i>) -> ValueResult<T>,
) -> ValueResult<T> {
    input
        .parse_nested_block(|block| read(block).map_err(ParseError::custom))
        .map_err(value_error)
}

pub(crate) fn comma_separated<'i, T>(
    input: &mut Parser<'i>,
    mut read_one: impl FnMut(&mut Parser<'i>) -> ValueResult<T>,
) -> ValueResult<Vec<T>> {
    input
        .parse_comma_separated(|item| read_one(item).map_err(ParseError::custom))
        .map_err(value_error)
}

fn value_error(error: ParseError<ValueError>) -> ValueError {
    match error.kind {
        ParseErrorKind::Custom(error) => error,
        ParseErrorKind::Basic(_) => ValueError::Mismatch,
    }
}

pub(crate) fn accepts_keyword(input: &mut Parser<'_>, keyword: &str) -> bool {
    input.try_parse(|input| input.expect_ident_matching(keyword)).is_ok()
}

/// Reads one of the keywords of `table`.
pub(crate) fn keyword<T: Copy>(input: &mut Parser<'_>, table: &[(&str, T)]) -> ValueResult<T> {
    let ident = input.expect_ident_cloned().map_err(|_| ValueError::Mismatch)?;
    table
        .iter()
        .find(|(name, _)| ident.eq_ignore_ascii_case(name))
        .map(|(_, value)| *value)
        .ok_or(ValueError::Mismatch)
}

pub(crate) fn non_negative_number(input: &mut Parser<'_>) -> ValueResult<f32> {
    match input.expect_number() {
        Ok(number) if number >= 0.0 && number.is_finite() => Ok(number),
        _ => Err(ValueError::Mismatch),
    }
}

/// Reads one to four values and spreads them over four slots as CSS does for sides and corners:
/// one value fills all, two alternate, three repeat the second in the last slot.
pub(crate) fn four_values<'i, T: Clone>(
    input: &mut Parser<'i>,
    mut read_one: impl FnMut(&mut Parser<'i>) -> ValueResult<T>,
) -> ValueResult<[T; 4]> {
    let mut values = vec![read_one(input)?];
    while values.len() < 4 && !input.is_exhausted() {
        values.push(read_one(input)?);
    }
    let slot = |index: usize| values[index].clone();
    Ok(match values.len() {
        1 => [slot(0), slot(0), slot(0), slot(0)],
        2 => [slot(0), slot(1), slot(0), slot(1)],
        3 => [slot(0), slot(1), slot(2), slot(1)],
        _ => [slot(0), slot(1), slot(2), slot(3)],
    })
}
