use std::borrow::Cow;

use cssparser::{Parser, Token};
use morok_style::{
    ColorStop, Corner, Gradient, GradientDirection, LengthPercentage, LinearGradient, Pixels, RadialExtent,
    RadialGradient, RadialShape,
};

use crate::compiled_tokens::CompiledTokens;
use crate::problem::Problem;
use crate::property_syntax::{accepts_keyword, color, comma_separated, keyword, length, nested, peek};
use crate::span::Span;
use crate::value_error::{ValueError, ValueResult};

const LINEAR_GRADIENT: &str = "linear-gradient";
const RADIAL_GRADIENT: &str = "radial-gradient";
const GRADIENT_SUFFIX: &str = "-gradient";
const DEGREES: &str = "deg";
const TO: &str = "to";
const AT: &str = "at";

const TOWARDS_BOTTOM: f32 = 180.0;
const CENTER: f32 = 0.5;

const SHAPES: [(&str, RadialShape); 2] = [("circle", RadialShape::Circle), ("ellipse", RadialShape::Ellipse)];
const EXTENTS: [(&str, RadialExtent); 4] = [
    ("closest-side", RadialExtent::ClosestSide),
    ("closest-corner", RadialExtent::ClosestCorner),
    ("farthest-side", RadialExtent::FarthestSide),
    ("farthest-corner", RadialExtent::FarthestCorner),
];
const EDGES: [(&str, Edge); 5] = [
    ("left", Edge::Left),
    ("right", Edge::Right),
    ("top", Edge::Top),
    ("bottom", Edge::Bottom),
    ("center", Edge::Center),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Edge {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

/// One coordinate of a gradient center: an edge keyword or a distance from the left or the top.
enum Coordinate {
    Edge(Edge),
    Offset(LengthPercentage),
}

pub(crate) fn is_ahead(input: &mut Parser<'_>) -> bool {
    matches!(peek(input), Some(Token::Function(name)) if name.to_ascii_lowercase().ends_with(GRADIENT_SUFFIX))
}

pub(crate) fn parse(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Gradient> {
    input.skip_whitespace();
    let start = input.position();
    let Token::Function(name) = input.next().map_err(|_| ValueError::Mismatch)?.clone() else {
        return Err(ValueError::Mismatch);
    };
    if name.eq_ignore_ascii_case(LINEAR_GRADIENT) {
        return nested(input, |arguments| linear(tokens, arguments)).map(Gradient::Linear);
    }
    if name.eq_ignore_ascii_case(RADIAL_GRADIENT) {
        return nested(input, |arguments| radial(tokens, arguments)).map(Gradient::Radial);
    }
    nested(input, |arguments| {
        while arguments.next().is_ok() {}
        Ok(())
    })?;
    Err(Problem {
        span: Span::between(start, input.position()),
        message: format!("`{name}()` is not supported"),
        hint: "use `linear-gradient()` or `radial-gradient()`".to_owned(),
    }
    .into())
}

fn linear(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<LinearGradient> {
    let direction = match direction(input)? {
        Some(direction) => {
            input.expect_comma().map_err(|_| ValueError::Mismatch)?;
            direction
        }
        None => GradientDirection::Angle(TOWARDS_BOTTOM),
    };
    Ok(LinearGradient {
        direction,
        stops: Cow::Owned(stops(tokens, input)?),
    })
}

/// The direction of a linear gradient when the arguments start with one.
fn direction(input: &mut Parser<'_>) -> ValueResult<Option<GradientDirection>> {
    input.skip_whitespace();
    let start = input.position();
    if let Some(Token::Dimension { value, unit, .. }) = peek(input) {
        input.next().map_err(|_| ValueError::Mismatch)?;
        if !unit.eq_ignore_ascii_case(DEGREES) {
            return Err(Problem {
                span: Span::between(start, input.position()),
                message: format!("the unit `{unit}` is not supported"),
                hint: "angles are in deg".to_owned(),
            }
            .into());
        }
        return Ok(Some(GradientDirection::Angle(value)));
    }
    if !accepts_keyword(input, TO) {
        return Ok(None);
    }
    let first = keyword(input, &EDGES)?;
    let second = input.try_parse(|input| keyword(input, &EDGES)).ok();
    Ok(Some(match (first, second) {
        (Edge::Top, None) => GradientDirection::Angle(0.0),
        (Edge::Right, None) => GradientDirection::Angle(90.0),
        (Edge::Bottom, None) => GradientDirection::Angle(TOWARDS_BOTTOM),
        (Edge::Left, None) => GradientDirection::Angle(270.0),
        (Edge::Top, Some(Edge::Left)) | (Edge::Left, Some(Edge::Top)) => GradientDirection::ToCorner(Corner::TopLeft),
        (Edge::Top, Some(Edge::Right)) | (Edge::Right, Some(Edge::Top)) => {
            GradientDirection::ToCorner(Corner::TopRight)
        }
        (Edge::Bottom, Some(Edge::Right)) | (Edge::Right, Some(Edge::Bottom)) => {
            GradientDirection::ToCorner(Corner::BottomRight)
        }
        (Edge::Bottom, Some(Edge::Left)) | (Edge::Left, Some(Edge::Bottom)) => {
            GradientDirection::ToCorner(Corner::BottomLeft)
        }
        _ => return Err(ValueError::Mismatch),
    }))
}

fn radial(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<RadialGradient> {
    let mut shape = None;
    let mut extent = None;
    let mut center = None;
    loop {
        input.skip_whitespace();
        let start = input.position();
        if shape.is_none()
            && let Ok(parsed) = input.try_parse(|input| keyword(input, &SHAPES))
        {
            shape = Some(parsed);
        } else if extent.is_none()
            && let Ok(parsed) = input.try_parse(|input| keyword(input, &EXTENTS))
        {
            extent = Some(parsed);
        } else if center.is_none() && accepts_keyword(input, AT) {
            center = Some(center_position(tokens, input)?);
        } else if matches!(peek(input), Some(Token::Dimension { .. } | Token::Percentage { .. })) {
            input.next().map_err(|_| ValueError::Mismatch)?;
            return Err(Problem {
                span: Span::between(start, input.position()),
                message: "an explicit radial gradient size is not supported".to_owned(),
                hint: "size it with `closest-side`, `closest-corner`, `farthest-side` or `farthest-corner`".to_owned(),
            }
            .into());
        } else {
            break;
        }
    }
    if shape.is_some() || extent.is_some() || center.is_some() {
        input.expect_comma().map_err(|_| ValueError::Mismatch)?;
    }
    let (center_x, center_y) = center.unwrap_or_else(|| (fraction(CENTER), fraction(CENTER)));
    Ok(RadialGradient {
        shape: shape.unwrap_or(RadialShape::Ellipse),
        extent: extent.unwrap_or(RadialExtent::FarthestCorner),
        center_x,
        center_y,
        stops: Cow::Owned(stops(tokens, input)?),
    })
}

/// The center after `at`: one or two coordinates, horizontal first unless the keywords say otherwise.
fn center_position(
    tokens: &CompiledTokens,
    input: &mut Parser<'_>,
) -> ValueResult<(LengthPercentage, LengthPercentage)> {
    let first = coordinate(tokens, input)?;
    let second = if matches!(peek(input), None | Some(Token::Comma)) {
        None
    } else {
        Some(coordinate(tokens, input)?)
    };
    let names_vertical_edge =
        |coordinate: &Coordinate| matches!(coordinate, Coordinate::Edge(Edge::Top | Edge::Bottom));
    let names_horizontal_edge =
        |coordinate: &Coordinate| matches!(coordinate, Coordinate::Edge(Edge::Left | Edge::Right));
    let (horizontal, vertical) = match (first, second) {
        (first, None) if names_vertical_edge(&first) => (Coordinate::Edge(Edge::Center), first),
        (first, None) => (first, Coordinate::Edge(Edge::Center)),
        (first, Some(second)) if names_vertical_edge(&first) || names_horizontal_edge(&second) => (second, first),
        (first, Some(second)) => (first, second),
    };
    let horizontal = match horizontal {
        Coordinate::Edge(Edge::Left) => fraction(0.0),
        Coordinate::Edge(Edge::Center) => fraction(CENTER),
        Coordinate::Edge(Edge::Right) => fraction(1.0),
        Coordinate::Offset(offset) => offset,
        Coordinate::Edge(Edge::Top | Edge::Bottom) => return Err(ValueError::Mismatch),
    };
    let vertical = match vertical {
        Coordinate::Edge(Edge::Top) => fraction(0.0),
        Coordinate::Edge(Edge::Center) => fraction(CENTER),
        Coordinate::Edge(Edge::Bottom) => fraction(1.0),
        Coordinate::Offset(offset) => offset,
        Coordinate::Edge(Edge::Left | Edge::Right) => return Err(ValueError::Mismatch),
    };
    Ok((horizontal, vertical))
}

fn coordinate(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Coordinate> {
    if let Ok(edge) = input.try_parse(|input| keyword(input, &EDGES)) {
        return Ok(Coordinate::Edge(edge));
    }
    Ok(Coordinate::Offset(length::length_percentage(tokens, input)?))
}

fn fraction(fraction: f32) -> LengthPercentage {
    LengthPercentage {
        pixels: Pixels(0.0),
        fraction: Some(fraction),
        tokens: Cow::Borrowed(&[]),
    }
}

/// Two or more color stops, each a color with an optional position.
fn stops(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Vec<ColorStop>> {
    let stops = comma_separated(input, |stop| {
        let color = color::parse(tokens, stop)?;
        let position = if stop.is_exhausted() {
            None
        } else {
            Some(length::length_percentage(tokens, stop)?)
        };
        Ok(ColorStop { color, position })
    })?;
    if stops.len() < 2 {
        return Err(ValueError::Mismatch);
    }
    Ok(stops)
}
