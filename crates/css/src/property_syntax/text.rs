use cssparser::{Parser, Token};
use morok_style::{
    Declaration, FontStyle, FontVariantNumeric, FontWeight, FontWeightValue, Length, LineHeight, Pixels, TextAlign,
    TextDecoration, TextOverflow, WhiteSpace,
};
use std::borrow::Cow;

use crate::compiled_tokens::CompiledTokens;
use crate::problem::Problem;
use crate::property::Property;
use crate::property_syntax::{accepts_keyword, keyword, length, peek};
use crate::span::Span;
use crate::token_reference;
use crate::value_error::{ValueError, ValueResult};

const NORMAL: &str = "normal";

const FONT_WEIGHT_KEYWORDS: [(&str, FontWeight); 2] = [("normal", FontWeight(400)), ("bold", FontWeight(700))];
const FONT_STYLES: [(&str, FontStyle); 2] = [("normal", FontStyle::Normal), ("italic", FontStyle::Italic)];
const TEXT_ALIGNMENTS: [(&str, TextAlign); 5] = [
    ("start", TextAlign::Start),
    ("end", TextAlign::End),
    ("left", TextAlign::Left),
    ("right", TextAlign::Right),
    ("center", TextAlign::Center),
];
const TEXT_DECORATIONS: [(&str, TextDecoration); 3] = [
    ("none", TextDecoration::None),
    ("underline", TextDecoration::Underline),
    ("line-through", TextDecoration::LineThrough),
];
const TEXT_OVERFLOWS: [(&str, TextOverflow); 2] = [("clip", TextOverflow::Clip), ("ellipsis", TextOverflow::Ellipsis)];
const WHITE_SPACES: [(&str, WhiteSpace); 4] = [
    ("normal", WhiteSpace::Normal),
    ("nowrap", WhiteSpace::Nowrap),
    ("pre", WhiteSpace::Pre),
    ("pre-wrap", WhiteSpace::PreWrap),
];
const NUMERIC_VARIANTS: [(&str, FontVariantNumeric); 2] = [
    ("normal", FontVariantNumeric::Normal),
    ("tabular-nums", FontVariantNumeric::TabularNums),
];

pub(crate) const PROPERTIES: &[Property] = &[
    Property {
        name: "font-family",
        expectation: "a `font-` token",
        parse: |tokens, input| Ok(vec![Declaration::FontFamily(font_family(tokens, input)?)]),
    },
    Property {
        name: "font-size",
        expectation: "a `text-` size token or a `calc()` over one, not negative",
        parse: |tokens, input| Ok(vec![Declaration::FontSize(font_size(tokens, input)?)]),
    },
    Property {
        name: "font-weight",
        expectation: "`normal`, `bold` or a `text-weight-` token",
        parse: |tokens, input| Ok(vec![Declaration::FontWeight(font_weight(tokens, input)?)]),
    },
    Property {
        name: "font-style",
        expectation: "`normal` or `italic`",
        parse: |_, input| Ok(vec![Declaration::FontStyle(keyword(input, &FONT_STYLES)?)]),
    },
    Property {
        name: "line-height",
        expectation: "`normal` or a `text-leading` token",
        parse: |tokens, input| Ok(vec![Declaration::LineHeight(line_height(tokens, input)?)]),
    },
    Property {
        name: "letter-spacing",
        expectation: "`normal` or a length",
        parse: |tokens, input| Ok(vec![Declaration::LetterSpacing(letter_spacing(tokens, input)?)]),
    },
    Property {
        name: "text-align",
        expectation: "`start`, `end`, `left`, `right` or `center`",
        parse: |_, input| Ok(vec![Declaration::TextAlign(keyword(input, &TEXT_ALIGNMENTS)?)]),
    },
    Property {
        name: "text-decoration",
        expectation: "`none`, `underline` or `line-through`",
        parse: |_, input| Ok(vec![Declaration::TextDecoration(keyword(input, &TEXT_DECORATIONS)?)]),
    },
    Property {
        name: "text-overflow",
        expectation: "`clip` or `ellipsis`",
        parse: |_, input| Ok(vec![Declaration::TextOverflow(keyword(input, &TEXT_OVERFLOWS)?)]),
    },
    Property {
        name: "white-space",
        expectation: "`normal`, `nowrap`, `pre` or `pre-wrap`",
        parse: |_, input| Ok(vec![Declaration::WhiteSpace(keyword(input, &WHITE_SPACES)?)]),
    },
    Property {
        name: "font-variant-numeric",
        expectation: "`normal` or `tabular-nums`",
        parse: |_, input| {
            Ok(vec![Declaration::FontVariantNumeric(keyword(
                input,
                &NUMERIC_VARIANTS,
            )?)])
        },
    },
];

/// Typography comes from tokens: a value written out in a stylesheet is reported with the
/// token family it belongs to.
fn raw_typography(input: &mut Parser<'_>, what: &str, token_family: &str) -> ValueError {
    input.skip_whitespace();
    let start = input.position();
    while input.next().is_ok() {}
    Problem {
        span: Span::between(start, input.position()),
        message: format!("raw {what} are not supported"),
        hint: format!("it comes from a `{token_family}` token: `var(--{token_family}…)`"),
    }
    .into()
}

fn font_family(
    tokens: &CompiledTokens,
    input: &mut Parser<'_>,
) -> ValueResult<morok_style::Token<morok_style::FontFamilies>> {
    if token_reference::is_ahead(input) {
        return token_reference::parse(tokens, input)?.font_families();
    }
    match peek(input) {
        Some(Token::Ident(_) | Token::QuotedString(_)) => Err(raw_typography(input, "font families", "font-")),
        _ => Err(ValueError::Mismatch),
    }
}

fn font_size(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Length> {
    let before = input.state();
    let size = length::non_negative_length(tokens, input)?;
    if size.tokens.is_empty() {
        input.reset(&before);
        return Err(raw_typography(input, "font sizes", "text-"));
    }
    Ok(size)
}

fn font_weight(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<FontWeightValue> {
    if token_reference::is_ahead(input) {
        return Ok(FontWeightValue::Token(
            token_reference::parse(tokens, input)?.font_weight()?,
        ));
    }
    if matches!(peek(input), Some(Token::Number { .. })) {
        return Err(raw_typography(input, "font weights", "text-weight-"));
    }
    Ok(FontWeightValue::Keyword(keyword(input, &FONT_WEIGHT_KEYWORDS)?))
}

fn line_height(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<LineHeight> {
    if token_reference::is_ahead(input) {
        return Ok(LineHeight::Token(token_reference::parse(tokens, input)?.leading()?));
    }
    if accepts_keyword(input, NORMAL) {
        return Ok(LineHeight::Normal);
    }
    match peek(input) {
        Some(Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. }) => {
            Err(raw_typography(input, "line heights", "text-leading"))
        }
        _ => Err(ValueError::Mismatch),
    }
}

fn letter_spacing(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Length> {
    if accepts_keyword(input, NORMAL) {
        return Ok(Length {
            pixels: Pixels(0.0),
            tokens: Cow::Borrowed(&[]),
        });
    }
    length::length(tokens, input)
}
