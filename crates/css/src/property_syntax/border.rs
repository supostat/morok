use std::borrow::Cow;

use cssparser::{Parser, Token};
use morok_style::{ColorValue, Corner, Declaration, Length, LineStyle, Pixels, Side};

use crate::compiled_tokens::CompiledTokens;
use crate::problem::Problem;
use crate::property::Property;
use crate::property_syntax::layout::on_every_side;
use crate::property_syntax::{color, four_values, keyword, length, peek};
use crate::span::Span;
use crate::value_error::{ValueError, ValueResult};

const LINE_STYLES: [(&str, LineStyle); 2] = [("none", LineStyle::None), ("solid", LineStyle::Solid)];

/// The CSS `medium` line width: the width of a border or an outline whose shorthand omits it.
const MEDIUM_LINE_WIDTH: f32 = 3.0;

const LINE: &str = "a width, a style (`solid` or `none`) and a color, in any order";
const WIDTH: &str = "a length, not negative";
const FOUR_WIDTHS: &str = "one to four lengths, not negative";
const STYLE: &str = "`solid` or `none`";
const FOUR_STYLES: &str = "one to four of `solid` and `none`";
const COLOR: &str = "a color token, `transparent` or `currentColor`";
const FOUR_COLORS: &str = "one to four color tokens, `transparent` or `currentColor`";
const RADIUS: &str = "a length, not negative";
const FOUR_RADII: &str = "one to four lengths, not negative";

/// The parts of a `border` or `outline` shorthand; an omitted part takes its initial value.
struct Line {
    width: Length,
    style: LineStyle,
    color: ColorValue,
}

pub(crate) const PROPERTIES: &[Property] = &[
    Property {
        name: "border",
        expectation: LINE,
        parse: |context, input| {
            let line = line(context.tokens, input)?;
            Ok(Side::ALL
                .into_iter()
                .flat_map(|side| border_side(side, &line))
                .collect())
        },
    },
    Property {
        name: "border-top",
        expectation: LINE,
        parse: |context, input| Ok(border_side(Side::Top, &line(context.tokens, input)?)),
    },
    Property {
        name: "border-right",
        expectation: LINE,
        parse: |context, input| Ok(border_side(Side::Right, &line(context.tokens, input)?)),
    },
    Property {
        name: "border-bottom",
        expectation: LINE,
        parse: |context, input| Ok(border_side(Side::Bottom, &line(context.tokens, input)?)),
    },
    Property {
        name: "border-left",
        expectation: LINE,
        parse: |context, input| Ok(border_side(Side::Left, &line(context.tokens, input)?)),
    },
    Property {
        name: "border-width",
        expectation: FOUR_WIDTHS,
        parse: |context, input| {
            let widths = four_values(input, |value| length::non_negative_length(context.tokens, value))?;
            Ok(on_every_side(widths, Declaration::BorderWidth))
        },
    },
    Property {
        name: "border-style",
        expectation: FOUR_STYLES,
        parse: |_, input| {
            let styles = four_values(input, |value| keyword(value, &LINE_STYLES))?;
            Ok(on_every_side(styles, Declaration::BorderStyle))
        },
    },
    Property {
        name: "border-color",
        expectation: FOUR_COLORS,
        parse: |context, input| {
            let colors = four_values(input, |value| color::parse(context.tokens, value))?;
            Ok(on_every_side(colors, Declaration::BorderColor))
        },
    },
    Property {
        name: "border-top-width",
        expectation: WIDTH,
        parse: |context, input| Ok(vec![border_width(Side::Top, context.tokens, input)?]),
    },
    Property {
        name: "border-right-width",
        expectation: WIDTH,
        parse: |context, input| Ok(vec![border_width(Side::Right, context.tokens, input)?]),
    },
    Property {
        name: "border-bottom-width",
        expectation: WIDTH,
        parse: |context, input| Ok(vec![border_width(Side::Bottom, context.tokens, input)?]),
    },
    Property {
        name: "border-left-width",
        expectation: WIDTH,
        parse: |context, input| Ok(vec![border_width(Side::Left, context.tokens, input)?]),
    },
    Property {
        name: "border-top-style",
        expectation: STYLE,
        parse: |_, input| Ok(vec![Declaration::BorderStyle(Side::Top, keyword(input, &LINE_STYLES)?)]),
    },
    Property {
        name: "border-right-style",
        expectation: STYLE,
        parse: |_, input| {
            Ok(vec![Declaration::BorderStyle(
                Side::Right,
                keyword(input, &LINE_STYLES)?,
            )])
        },
    },
    Property {
        name: "border-bottom-style",
        expectation: STYLE,
        parse: |_, input| {
            Ok(vec![Declaration::BorderStyle(
                Side::Bottom,
                keyword(input, &LINE_STYLES)?,
            )])
        },
    },
    Property {
        name: "border-left-style",
        expectation: STYLE,
        parse: |_, input| {
            Ok(vec![Declaration::BorderStyle(
                Side::Left,
                keyword(input, &LINE_STYLES)?,
            )])
        },
    },
    Property {
        name: "border-top-color",
        expectation: COLOR,
        parse: |context, input| {
            Ok(vec![Declaration::BorderColor(
                Side::Top,
                color::parse(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "border-right-color",
        expectation: COLOR,
        parse: |context, input| {
            Ok(vec![Declaration::BorderColor(
                Side::Right,
                color::parse(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "border-bottom-color",
        expectation: COLOR,
        parse: |context, input| {
            Ok(vec![Declaration::BorderColor(
                Side::Bottom,
                color::parse(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "border-left-color",
        expectation: COLOR,
        parse: |context, input| {
            Ok(vec![Declaration::BorderColor(
                Side::Left,
                color::parse(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "border-radius",
        expectation: FOUR_RADII,
        parse: |context, input| {
            let radii = four_values(input, |value| radius(context.tokens, value))?;
            Ok(Corner::ALL
                .into_iter()
                .zip(radii)
                .map(|(corner, radius)| Declaration::BorderRadius(corner, radius))
                .collect())
        },
    },
    Property {
        name: "border-top-left-radius",
        expectation: RADIUS,
        parse: |context, input| {
            Ok(vec![Declaration::BorderRadius(
                Corner::TopLeft,
                radius(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "border-top-right-radius",
        expectation: RADIUS,
        parse: |context, input| {
            Ok(vec![Declaration::BorderRadius(
                Corner::TopRight,
                radius(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "border-bottom-right-radius",
        expectation: RADIUS,
        parse: |context, input| {
            Ok(vec![Declaration::BorderRadius(
                Corner::BottomRight,
                radius(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "border-bottom-left-radius",
        expectation: RADIUS,
        parse: |context, input| {
            Ok(vec![Declaration::BorderRadius(
                Corner::BottomLeft,
                radius(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "outline",
        expectation: LINE,
        parse: |context, input| {
            let line = line(context.tokens, input)?;
            Ok(vec![
                Declaration::OutlineWidth(line.width),
                Declaration::OutlineStyle(line.style),
                Declaration::OutlineColor(line.color),
            ])
        },
    },
    Property {
        name: "outline-width",
        expectation: WIDTH,
        parse: |context, input| {
            Ok(vec![Declaration::OutlineWidth(length::non_negative_length(
                context.tokens,
                input,
            )?)])
        },
    },
    Property {
        name: "outline-style",
        expectation: STYLE,
        parse: |_, input| Ok(vec![Declaration::OutlineStyle(keyword(input, &LINE_STYLES)?)]),
    },
    Property {
        name: "outline-color",
        expectation: COLOR,
        parse: |context, input| Ok(vec![Declaration::OutlineColor(color::parse(context.tokens, input)?)]),
    },
    Property {
        name: "outline-offset",
        expectation: "a length",
        parse: |context, input| Ok(vec![Declaration::OutlineOffset(length::length(context.tokens, input)?)]),
    },
];

fn border_width(side: Side, tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Declaration> {
    Ok(Declaration::BorderWidth(
        side,
        length::non_negative_length(tokens, input)?,
    ))
}

fn border_side(side: Side, line: &Line) -> Vec<Declaration> {
    vec![
        Declaration::BorderWidth(side, line.width.clone()),
        Declaration::BorderStyle(side, line.style),
        Declaration::BorderColor(side, line.color),
    ]
}

/// One corner radius. Two radii after `/` would make the corner elliptical, which the subset rejects.
fn radius(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Length> {
    let radius = length::non_negative_length(tokens, input)?;
    input.skip_whitespace();
    let start = input.position();
    if let Some(Token::Delim('/')) = peek(input) {
        while input.next().is_ok() {}
        return Err(Problem {
            span: Span::between(start, input.position()),
            message: "elliptical corners are not supported".to_owned(),
            hint: "give each corner one radius".to_owned(),
        }
        .into());
    }
    Ok(radius)
}

fn line(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Line> {
    let mut width = None;
    let mut style = None;
    let mut line_color = None;
    while !input.is_exhausted() {
        if style.is_none()
            && let Ok(parsed) = input.try_parse(|input| keyword(input, &LINE_STYLES))
        {
            style = Some(parsed);
        } else if line_color.is_none() && color::is_ahead(tokens, input)? {
            line_color = Some(color::parse(tokens, input)?);
        } else if width.is_none() {
            width = Some(length::non_negative_length(tokens, input)?);
        } else {
            return Err(ValueError::Mismatch);
        }
    }
    if width.is_none() && style.is_none() && line_color.is_none() {
        return Err(ValueError::Mismatch);
    }
    Ok(Line {
        width: width.unwrap_or(Length {
            pixels: Pixels(MEDIUM_LINE_WIDTH),
            tokens: Cow::Borrowed(&[]),
        }),
        style: style.unwrap_or(LineStyle::None),
        color: line_color.unwrap_or(ColorValue::CurrentColor),
    })
}
