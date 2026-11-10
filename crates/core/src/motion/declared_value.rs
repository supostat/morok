use morok_style::{BoxShadow, Color, ColorValue, Declaration, Theme};

use crate::motion::property_value::PropertyValue;
use crate::motion::transform_step::{TransformStep, resolve_into};
use crate::style::paint_style::TRANSPARENT;
use crate::style::resolved_length::ResolvedLength;
use crate::style::resolved_length_or_auto::ResolvedLengthOrAuto;

/// The value an animatable declaration gives its property, its tokens resolved against the
/// theme and `currentColor` taken as `current_color`. A transform is resolved into `steps`.
///
/// # Panics
///
/// When the declaration is of a property that is not animatable: a keyframe holds none.
pub(crate) fn declared_value<'a>(
    declaration: &'a Declaration,
    theme: &'a Theme,
    current_color: Color,
    steps: &'a mut Vec<TransformStep>,
) -> PropertyValue<'a> {
    let color = |value: &ColorValue| match value {
        ColorValue::Token(token) => *theme.resolve(*token),
        ColorValue::Transparent => TRANSPARENT,
        ColorValue::CurrentColor => current_color,
    };
    match declaration {
        Declaration::Width(size)
        | Declaration::Height(size)
        | Declaration::MinWidth(size)
        | Declaration::MinHeight(size)
        | Declaration::MaxWidth(size)
        | Declaration::MaxHeight(size)
        | Declaration::Margin(_, size)
        | Declaration::Inset(_, size) => PropertyValue::Size(ResolvedLengthOrAuto::of(size, theme)),
        Declaration::Padding(_, length) | Declaration::RowGap(length) | Declaration::ColumnGap(length) => {
            PropertyValue::Length(ResolvedLength::of(length, theme))
        }
        Declaration::BorderWidth(_, length)
        | Declaration::BorderRadius(_, length)
        | Declaration::OutlineWidth(length)
        | Declaration::OutlineOffset(length) => PropertyValue::Pixels(ResolvedLength::pixels_of(length, theme)),
        Declaration::BorderColor(_, value)
        | Declaration::OutlineColor(value)
        | Declaration::Color(value)
        | Declaration::BackgroundColor(value) => PropertyValue::Color(color(value)),
        Declaration::Opacity(opacity) => PropertyValue::Opacity(opacity.clamp(0.0, 1.0)),
        Declaration::BoxShadow(BoxShadow::None) => PropertyValue::Shadow(&[]),
        Declaration::BoxShadow(BoxShadow::Token(token)) => PropertyValue::Shadow(&theme.resolve(*token).layers),
        Declaration::Transform(functions) => {
            resolve_into(functions, theme, steps);
            PropertyValue::Transform(steps)
        }
        other => panic!("{other:?} is not animatable"),
    }
}
