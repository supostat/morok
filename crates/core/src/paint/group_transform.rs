use morok_style::{Declaration, StyleSheet, Theme, TransformFunction};

use crate::element::Element;
use crate::geometry::transform::Transform;
use crate::style::resolved_length::ResolvedLength;

/// The transform of an element in the coordinates it was laid out in: the functions of its
/// `transform` list applied in order about its `transform-origin`.
pub(crate) fn local_transform(element: &Element, style_sheet: &StyleSheet, theme: &Theme) -> Transform {
    let Some(source) = element.style.effects.transform else {
        return Transform::IDENTITY;
    };
    let Declaration::Transform(functions) = source.declaration(style_sheet, &element.inline_style) else {
        unreachable!("the transform of a style points at a transform declaration");
    };
    let border_box = &element.border_box;
    let mut functions_applied = Transform::IDENTITY;
    for function in functions.iter() {
        let matrix = match function {
            TransformFunction::Translate(x, y) => Transform::translation(
                ResolvedLength::of(x, theme).against(border_box.width),
                ResolvedLength::of(y, theme).against(border_box.height),
            ),
            TransformFunction::TranslateX(x) => {
                Transform::translation(ResolvedLength::of(x, theme).against(border_box.width), 0.0)
            }
            TransformFunction::TranslateY(y) => {
                Transform::translation(0.0, ResolvedLength::of(y, theme).against(border_box.height))
            }
            TransformFunction::Scale(x, y) => Transform::scaling(*x, *y),
            TransformFunction::ScaleX(x) => Transform::scaling(*x, 1.0),
            TransformFunction::ScaleY(y) => Transform::scaling(1.0, *y),
            TransformFunction::Rotate(degrees) => Transform::rotation(*degrees),
        };
        functions_applied = matrix.then(&functions_applied);
    }
    let origin_x = border_box.x + element.style.effects.transform_origin_x.against(border_box.width);
    let origin_y = border_box.y + element.style.effects.transform_origin_y.against(border_box.height);
    Transform::translation(-origin_x, -origin_y)
        .then(&functions_applied)
        .then(&Transform::translation(origin_x, origin_y))
}
