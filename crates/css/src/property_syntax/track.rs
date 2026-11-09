use std::borrow::Cow;

use cssparser::{Parser, Token};
use morok_style::{RepeatCount, TrackBreadth, TrackListItem, TrackSize};

use crate::compiled_tokens::CompiledTokens;
use crate::problem::Problem;
use crate::property_syntax::{accepts_keyword, keyword, length, nested, peek};
use crate::span::Span;
use crate::value_error::{ValueError, ValueResult};

const FLEX_UNIT: &str = "fr";
const NONE: &str = "none";
const MINMAX: &str = "minmax";
const FIT_CONTENT: &str = "fit-content";
const REPEAT: &str = "repeat";
const CONTENT_BREADTHS: [(&str, ContentBreadth); 3] = [
    ("auto", ContentBreadth::Auto),
    ("min-content", ContentBreadth::MinContent),
    ("max-content", ContentBreadth::MaxContent),
];
const AUTOMATIC_REPEATS: [(&str, RepeatCount); 2] =
    [("auto-fill", RepeatCount::AutoFill), ("auto-fit", RepeatCount::AutoFit)];

#[derive(Clone, Copy)]
enum ContentBreadth {
    Auto,
    MinContent,
    MaxContent,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Flexibility {
    Flexible,
    Inflexible,
}

/// The tracks of `grid-template-columns` or `grid-template-rows`; `none` is no tracks.
pub(crate) fn template(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Vec<TrackListItem>> {
    if accepts_keyword(input, NONE) {
        return Ok(Vec::new());
    }
    let mut items = vec![template_item(tokens, input)?];
    while !input.is_exhausted() {
        items.push(template_item(tokens, input)?);
    }
    Ok(items)
}

/// The tracks of `grid-auto-columns` or `grid-auto-rows`: one or more sizes.
pub(crate) fn sizes(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Vec<TrackSize>> {
    let mut sizes = vec![size(tokens, input)?];
    while !input.is_exhausted() {
        sizes.push(size(tokens, input)?);
    }
    Ok(sizes)
}

fn template_item(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<TrackListItem> {
    input.skip_whitespace();
    let start = input.position();
    match peek(input) {
        Some(Token::SquareBracketBlock) => {
            input.next().map_err(|_| ValueError::Mismatch)?;
            nested(input, |names| {
                while names.next().is_ok() {}
                Ok(())
            })?;
            Err(Problem {
                span: Span::between(start, input.position()),
                message: "named grid lines are not supported".to_owned(),
                hint: "place items by line number or by `span`".to_owned(),
            }
            .into())
        }
        Some(Token::Function(name)) if name.eq_ignore_ascii_case(REPEAT) => {
            input.next().map_err(|_| ValueError::Mismatch)?;
            nested(input, |arguments| repetition(tokens, arguments))
        }
        _ => Ok(TrackListItem::Track(size(tokens, input)?)),
    }
}

fn repetition(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<TrackListItem> {
    let count = if let Ok(automatic) = input.try_parse(|input| keyword(input, &AUTOMATIC_REPEATS)) {
        automatic
    } else {
        let times = input.expect_integer().map_err(|_| ValueError::Mismatch)?;
        match u16::try_from(times) {
            Ok(times) if times > 0 => RepeatCount::Times(times),
            _ => return Err(ValueError::Mismatch),
        }
    };
    input.expect_comma().map_err(|_| ValueError::Mismatch)?;
    Ok(TrackListItem::Repeat {
        count,
        tracks: Cow::Owned(sizes(tokens, input)?),
    })
}

fn size(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<TrackSize> {
    match peek(input) {
        Some(Token::Function(name)) if name.eq_ignore_ascii_case(MINMAX) => {
            input.next().map_err(|_| ValueError::Mismatch)?;
            nested(input, |arguments| {
                let min = breadth(Flexibility::Inflexible, tokens, arguments)?;
                arguments.expect_comma().map_err(|_| ValueError::Mismatch)?;
                let max = breadth(Flexibility::Flexible, tokens, arguments)?;
                Ok(TrackSize::MinMax { min, max })
            })
        }
        Some(Token::Function(name)) if name.eq_ignore_ascii_case(FIT_CONTENT) => {
            input.next().map_err(|_| ValueError::Mismatch)?;
            nested(input, |arguments| {
                Ok(TrackSize::FitContent(length::non_negative_length_percentage(
                    tokens, arguments,
                )?))
            })
        }
        _ => Ok(TrackSize::Breadth(breadth(Flexibility::Flexible, tokens, input)?)),
    }
}

fn breadth(flexibility: Flexibility, tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<TrackBreadth> {
    if let Ok(content) = input.try_parse(|input| keyword(input, &CONTENT_BREADTHS)) {
        return Ok(match content {
            ContentBreadth::Auto => TrackBreadth::Auto,
            ContentBreadth::MinContent => TrackBreadth::MinContent,
            ContentBreadth::MaxContent => TrackBreadth::MaxContent,
        });
    }
    if let Some(Token::Dimension { value, unit, .. }) = peek(input)
        && unit.eq_ignore_ascii_case(FLEX_UNIT)
    {
        if flexibility == Flexibility::Inflexible || value < 0.0 || !value.is_finite() {
            return Err(ValueError::Mismatch);
        }
        input.next().map_err(|_| ValueError::Mismatch)?;
        return Ok(TrackBreadth::Flex(value));
    }
    Ok(TrackBreadth::LengthPercentage(length::non_negative_length_percentage(
        tokens, input,
    )?))
}
