use std::f32::consts::{PI, TAU};
use std::fmt::Write;

use morok_core::Rect;
use morok_css::{CompiledTokens, TokenId};
use morok_platform::RingSector;

use crate::canvas_outline::CanvasOutline;
use crate::canvas_shape::CanvasShape;

/// The SVG paths a canvas holds on the page: one per shape, filled with its token, so that the
/// page picks the value of the token under its color scheme itself. A path is drawn in the
/// CSS pixels of the content box of the canvas, as the kit draws it.
///
/// # Panics
///
/// When a shape is colored by a token the tokens of the suite do not have.
pub(crate) fn paths(shapes: &[CanvasShape], tokens: &CompiledTokens) -> String {
    let mut paths = String::new();
    for shape in shapes {
        let token = tokens
            .entries
            .iter()
            .find(|entry| entry.id == TokenId::Color(shape.color))
            .unwrap_or_else(|| panic!("the tokens of the suite have the color {:?}", shape.color));
        let (outline, fill_rule) = match &shape.outline {
            CanvasOutline::Ring(sector) => ring_outline(sector),
            CanvasOutline::Rectangle(rect) => (rectangle_outline(rect), "nonzero"),
        };
        let _ = write!(
            paths,
            "<path d=\"{outline}\" fill-rule=\"{fill_rule}\" style=\"fill: var(--{}); fill-opacity: {}\"></path>",
            token.name, shape.opacity
        );
    }
    paths
}

/// The outline of the rectangle, clockwise from its top left corner.
fn rectangle_outline(rect: &Rect) -> String {
    format!(
        "M{:.3} {:.3} H{:.3} V{:.3} H{:.3} Z",
        rect.x,
        rect.y,
        rect.right(),
        rect.bottom(),
        rect.x
    )
}

/// The outline of the sector and the rule that fills it. A sector of a whole turn or more is
/// the ring between two circles; any other runs along the outer arc, back along the inner one,
/// and closes along its two radii.
fn ring_outline(sector: &RingSector) -> (String, &'static str) {
    let RingSector {
        center,
        inner_radius,
        outer_radius,
        start_angle,
        end_angle,
    } = *sector;
    let point = |radius: f32, angle: f32| {
        format!(
            "{:.3} {:.3}",
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin()
        )
    };
    let span = end_angle - start_angle;
    if span >= TAU {
        let circle = |radius: f32| {
            format!(
                "M{} A{radius} {radius} 0 1 1 {} A{radius} {radius} 0 1 1 {} Z",
                point(radius, 0.0),
                point(radius, PI),
                point(radius, 0.0)
            )
        };
        return (format!("{} {}", circle(outer_radius), circle(inner_radius)), "evenodd");
    }
    let is_large = u8::from(span > PI);
    (
        format!(
            "M{} A{outer_radius} {outer_radius} 0 {is_large} 1 {} L{} A{inner_radius} {inner_radius} 0 {is_large} 0 {} Z",
            point(outer_radius, start_angle),
            point(outer_radius, end_angle),
            point(inner_radius, end_angle),
            point(inner_radius, start_angle)
        ),
        "nonzero",
    )
}
