use std::borrow::Cow;

use cssparser::{Parser, Token};
use morok_style::{
    BackdropFilterFunction, BlendMode, Declaration, FilterFunction, LengthPercentage, TransformFunction,
};

use crate::compiled_tokens::CompiledTokens;
use crate::linear::Linear;
use crate::problem::Problem;
use crate::property::Property;
use crate::property_syntax::{accepts_keyword, keyword, length, nested, peek};
use crate::span::Span;
use crate::value_error::{ValueError, ValueResult};

const NONE: &str = "none";
const DEGREES: &str = "deg";
const CENTER: f32 = 0.5;

const TRANSFORM_FUNCTIONS: [&str; 7] = [
    "translate",
    "translateX",
    "translateY",
    "scale",
    "scaleX",
    "scaleY",
    "rotate",
];
const FILTER_FUNCTIONS: [&str; 6] = ["blur", "brightness", "contrast", "grayscale", "saturate", "opacity"];
const BACKDROP_FILTER_FUNCTIONS: [&str; 3] = ["blur", "brightness", "saturate"];
const BLEND_MODES: [(&str, BlendMode); 6] = [
    ("normal", BlendMode::Normal),
    ("multiply", BlendMode::Multiply),
    ("screen", BlendMode::Screen),
    ("overlay", BlendMode::Overlay),
    ("darken", BlendMode::Darken),
    ("lighten", BlendMode::Lighten),
];
const HORIZONTAL_EDGES: [(&str, f32); 3] = [("left", 0.0), ("center", CENTER), ("right", 1.0)];
const VERTICAL_EDGES: [(&str, f32); 3] = [("top", 0.0), ("center", CENTER), ("bottom", 1.0)];

pub(crate) const PROPERTIES: &[Property] = &[
    Property {
        name: "opacity",
        expectation: "a number or a percentage",
        parse: |_, input| Ok(vec![Declaration::Opacity(unit_amount(input)?)]),
    },
    Property {
        name: "transform",
        expectation: "`none` or a list of `translate()`, `scale()`, `rotate()` and their one-axis forms",
        parse: |context, input| Ok(vec![Declaration::Transform(transform(context.tokens, input)?)]),
    },
    Property {
        name: "transform-origin",
        expectation: "one or two lengths, percentages or edge keywords",
        parse: |context, input| {
            let (horizontal, vertical) = transform_origin(context.tokens, input)?;
            Ok(vec![Declaration::TransformOrigin(horizontal, vertical)])
        },
    },
    Property {
        name: "filter",
        expectation: "`none` or a list of `blur()`, `brightness()`, `contrast()`, `grayscale()`, `saturate()` and `opacity()`",
        parse: |context, input| Ok(vec![Declaration::Filter(filter(context.tokens, input)?)]),
    },
    Property {
        name: "backdrop-filter",
        expectation: "`none` or a list of `blur()`, `brightness()` and `saturate()`",
        parse: |context, input| {
            Ok(vec![Declaration::BackdropFilter(backdrop_filter(
                context.tokens,
                input,
            )?)])
        },
    },
    Property {
        name: "mix-blend-mode",
        expectation: "`normal`, `multiply`, `screen`, `overlay`, `darken` or `lighten`",
        parse: |_, input| Ok(vec![Declaration::MixBlendMode(keyword(input, &BLEND_MODES)?)]),
    },
];

/// A list of functions, or `none` for no function at all.
fn function_list<'i, T>(
    input: &mut Parser<'i>,
    mut read_one: impl FnMut(&mut Parser<'i>) -> ValueResult<T>,
) -> ValueResult<Cow<'static, [T]>>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    if accepts_keyword(input, NONE) {
        return Ok(Cow::Borrowed(&[]));
    }
    let mut functions = vec![read_one(input)?];
    while !input.is_exhausted() {
        functions.push(read_one(input)?);
    }
    Ok(Cow::Owned(functions))
}

/// The name of the function ahead, as written, with its span once the arguments are consumed.
struct FunctionCall<'i> {
    name: cssparser::CowRcStr<'i>,
    start: cssparser::SourcePosition,
}

fn function_ahead<'i>(input: &mut Parser<'i>) -> ValueResult<FunctionCall<'i>> {
    input.skip_whitespace();
    let start = input.position();
    match input.next().map_err(|_| ValueError::Mismatch)?.clone() {
        Token::Function(name) => Ok(FunctionCall { name, start }),
        _ => Err(ValueError::Mismatch),
    }
}

/// A function outside the list of the property: its arguments are skipped and the call is
/// reported with the functions that are allowed.
fn unsupported_function(
    input: &mut Parser<'_>,
    call: &FunctionCall<'_>,
    property: &str,
    allowed: &[&str],
) -> ValueError {
    let skipped = nested(input, |arguments| {
        while arguments.next().is_ok() {}
        Ok(())
    });
    if let Err(error) = skipped {
        return error;
    }
    let allowed: Vec<String> = allowed.iter().map(|name| format!("`{name}()`")).collect();
    Problem {
        span: Span::between(call.start, input.position()),
        message: format!("`{}()` is not supported in `{property}`", call.name),
        hint: format!("the functions of `{property}` are {}", allowed.join(", ")),
    }
    .into()
}

fn fraction_of_the_box(fraction: f32) -> LengthPercentage {
    Linear::fraction(fraction).into_length_percentage()
}

/// A factor as CSS filter functions take it: a number, or a percentage of the whole.
fn amount(input: &mut Parser<'_>) -> ValueResult<f32> {
    match input.next().map_err(|_| ValueError::Mismatch)?.clone() {
        Token::Number { value, .. } if value >= 0.0 && value.is_finite() => Ok(value),
        Token::Percentage { unit_value, .. } if unit_value >= 0.0 && unit_value.is_finite() => Ok(unit_value),
        _ => Err(ValueError::Mismatch),
    }
}

/// A factor between nothing and the whole, as `opacity` and `grayscale()` take it; more is clamped.
fn unit_amount(input: &mut Parser<'_>) -> ValueResult<f32> {
    Ok(amount(input)?.min(1.0))
}

fn number(input: &mut Parser<'_>) -> ValueResult<f32> {
    match input.expect_number() {
        Ok(value) if value.is_finite() => Ok(value),
        _ => Err(ValueError::Mismatch),
    }
}

fn degrees(input: &mut Parser<'_>) -> ValueResult<f32> {
    input.skip_whitespace();
    let start = input.position();
    match input.next().map_err(|_| ValueError::Mismatch)?.clone() {
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case(DEGREES) && value.is_finite() => Ok(value),
        Token::Dimension { unit, .. } => Err(Problem {
            span: Span::between(start, input.position()),
            message: format!("the unit `{unit}` is not supported"),
            hint: "angles are in deg".to_owned(),
        }
        .into()),
        Token::Number { value: 0.0, .. } => Ok(0.0),
        _ => Err(ValueError::Mismatch),
    }
}

fn transform(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Cow<'static, [TransformFunction]>> {
    function_list(input, |input| transform_function(tokens, input))
}

fn transform_function(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<TransformFunction> {
    let call = function_ahead(input)?;
    let name = call.name.as_ref();
    let two_lengths = |input: &mut Parser<'_>| -> ValueResult<(LengthPercentage, LengthPercentage)> {
        let first = length::length_percentage(tokens, input)?;
        let second = if input.try_parse(Parser::expect_comma).is_ok() {
            length::length_percentage(tokens, input)?
        } else {
            Linear::pixels(0.0).into_length_percentage()
        };
        Ok((first, second))
    };
    let two_numbers = |input: &mut Parser<'_>| -> ValueResult<(f32, f32)> {
        let first = number(input)?;
        let second = if input.try_parse(Parser::expect_comma).is_ok() {
            number(input)?
        } else {
            first
        };
        Ok((first, second))
    };
    let function = match name {
        "translate" => nested(input, |arguments| {
            let (x, y) = two_lengths(arguments)?;
            Ok(TransformFunction::Translate(x, y))
        })?,
        "translateX" => nested(input, |arguments| {
            Ok(TransformFunction::TranslateX(length::length_percentage(
                tokens, arguments,
            )?))
        })?,
        "translateY" => nested(input, |arguments| {
            Ok(TransformFunction::TranslateY(length::length_percentage(
                tokens, arguments,
            )?))
        })?,
        "scale" => nested(input, |arguments| {
            let (x, y) = two_numbers(arguments)?;
            Ok(TransformFunction::Scale(x, y))
        })?,
        "scaleX" => nested(input, |arguments| Ok(TransformFunction::ScaleX(number(arguments)?)))?,
        "scaleY" => nested(input, |arguments| Ok(TransformFunction::ScaleY(number(arguments)?)))?,
        "rotate" => nested(input, |arguments| Ok(TransformFunction::Rotate(degrees(arguments)?)))?,
        _ => return Err(unsupported_function(input, &call, "transform", &TRANSFORM_FUNCTIONS)),
    };
    Ok(function)
}

/// One or two coordinates: lengths, percentages or edge keywords, horizontal first unless the
/// keywords say otherwise. One coordinate leaves the other at the center.
fn transform_origin(
    tokens: &CompiledTokens,
    input: &mut Parser<'_>,
) -> ValueResult<(LengthPercentage, LengthPercentage)> {
    let first = origin_coordinate(tokens, input)?;
    if input.is_exhausted() {
        return Ok(if first.is_vertical() {
            (fraction_of_the_box(CENTER), first.length())
        } else {
            (first.length(), fraction_of_the_box(CENTER))
        });
    }
    let second = origin_coordinate(tokens, input)?;
    let (horizontal, vertical) = if first.is_vertical() {
        (second, first)
    } else {
        (first, second)
    };
    if horizontal.is_vertical() || vertical.is_horizontal() {
        return Err(ValueError::Mismatch);
    }
    Ok((horizontal.length(), vertical.length()))
}

/// A coordinate of the origin and the axis its spelling binds it to.
enum Coordinate {
    Horizontal(LengthPercentage),
    Vertical(LengthPercentage),
    /// A length, a percentage or `center`: either axis.
    Either(LengthPercentage),
}

impl Coordinate {
    fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal(_))
    }

    fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical(_))
    }

    fn length(self) -> LengthPercentage {
        match self {
            Self::Horizontal(length) | Self::Vertical(length) | Self::Either(length) => length,
        }
    }
}

fn origin_coordinate(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Coordinate> {
    if let Some(Token::Ident(name)) = peek(input) {
        if name.eq_ignore_ascii_case("center") {
            input.next().map_err(|_| ValueError::Mismatch)?;
            return Ok(Coordinate::Either(fraction_of_the_box(CENTER)));
        }
        if let Ok(fraction) = input.try_parse(|input| keyword(input, &HORIZONTAL_EDGES)) {
            return Ok(Coordinate::Horizontal(fraction_of_the_box(fraction)));
        }
        if let Ok(fraction) = input.try_parse(|input| keyword(input, &VERTICAL_EDGES)) {
            return Ok(Coordinate::Vertical(fraction_of_the_box(fraction)));
        }
        return Err(ValueError::Mismatch);
    }
    Ok(Coordinate::Either(length::length_percentage(tokens, input)?))
}

fn filter(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Cow<'static, [FilterFunction]>> {
    function_list(input, |input| {
        let call = function_ahead(input)?;
        let function = match call.name.as_ref() {
            "blur" => nested(input, |arguments| {
                Ok(FilterFunction::Blur(length::non_negative_length(tokens, arguments)?))
            })?,
            "brightness" => nested(input, |arguments| Ok(FilterFunction::Brightness(amount(arguments)?)))?,
            "contrast" => nested(input, |arguments| Ok(FilterFunction::Contrast(amount(arguments)?)))?,
            "grayscale" => nested(input, |arguments| {
                Ok(FilterFunction::Grayscale(unit_amount(arguments)?))
            })?,
            "saturate" => nested(input, |arguments| Ok(FilterFunction::Saturate(amount(arguments)?)))?,
            "opacity" => nested(input, |arguments| Ok(FilterFunction::Opacity(unit_amount(arguments)?)))?,
            _ => return Err(unsupported_function(input, &call, "filter", &FILTER_FUNCTIONS)),
        };
        Ok(function)
    })
}

fn backdrop_filter(
    tokens: &CompiledTokens,
    input: &mut Parser<'_>,
) -> ValueResult<Cow<'static, [BackdropFilterFunction]>> {
    function_list(input, |input| {
        let call = function_ahead(input)?;
        let function = match call.name.as_ref() {
            "blur" => nested(input, |arguments| {
                Ok(BackdropFilterFunction::Blur(length::non_negative_length(
                    tokens, arguments,
                )?))
            })?,
            "brightness" => nested(input, |arguments| {
                Ok(BackdropFilterFunction::Brightness(amount(arguments)?))
            })?,
            "saturate" => nested(input, |arguments| {
                Ok(BackdropFilterFunction::Saturate(amount(arguments)?))
            })?,
            _ => {
                return Err(unsupported_function(
                    input,
                    &call,
                    "backdrop-filter",
                    &BACKDROP_FILTER_FUNCTIONS,
                ));
            }
        };
        Ok(function)
    })
}
