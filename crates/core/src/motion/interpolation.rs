use morok_style::{AnimatableProperty, Color, Pixels, ShadowLayer};

use crate::motion::held_value::HeldValue;
use crate::motion::property_value::PropertyValue;
use crate::motion::transform_step::blend_into;
use crate::style::resolved_length::ResolvedLength;
use crate::style::resolved_length_or_auto::ResolvedLengthOrAuto;

const TRANSPARENT: Color = Color {
    red: 0,
    green: 0,
    blue: 0,
    alpha: 0.0,
};

/// Whether the two values have values in between: `auto` has none with a length, and two
/// shadows have none when a pair of their layers differs in `inset`.
pub(crate) fn is_interpolable(from: PropertyValue<'_>, to: PropertyValue<'_>) -> bool {
    match (from, to) {
        (PropertyValue::Size(ResolvedLengthOrAuto::Auto), PropertyValue::Size(ResolvedLengthOrAuto::Length(_)))
        | (PropertyValue::Size(ResolvedLengthOrAuto::Length(_)), PropertyValue::Size(ResolvedLengthOrAuto::Auto)) => {
            false
        }
        (PropertyValue::Shadow(from), PropertyValue::Shadow(to)) => {
            from.iter().zip(to).all(|(from, to)| from.inset == to.inset)
        }
        _ => true,
    }
}

/// Writes into `value` the value of the property `progress` of the way from `from` to `to`. A
/// pair without values in between flips half way. A value the property cannot take, as an
/// overshooting easing reaches, is clamped to the nearest it can.
pub(crate) fn interpolate(
    property: AnimatableProperty,
    from: PropertyValue<'_>,
    to: PropertyValue<'_>,
    progress: f64,
    value: &mut HeldValue,
) {
    if !is_interpolable(from, to) {
        value.set(if progress < 0.5 { from } else { to });
        return;
    }
    let lower_bound = if is_non_negative(property) {
        0.0
    } else {
        f32::NEG_INFINITY
    };
    let fraction = progress as f32;
    match (from, to) {
        (
            PropertyValue::Size(ResolvedLengthOrAuto::Length(from)),
            PropertyValue::Size(ResolvedLengthOrAuto::Length(to)),
        ) => {
            value.set(PropertyValue::Size(ResolvedLengthOrAuto::Length(lengths_between(
                from,
                to,
                fraction,
                lower_bound,
            ))));
        }
        (PropertyValue::Length(from), PropertyValue::Length(to)) => {
            value.set(PropertyValue::Length(lengths_between(from, to, fraction, lower_bound)));
        }
        (PropertyValue::Pixels(from), PropertyValue::Pixels(to)) => {
            value.set(PropertyValue::Pixels(between(from, to, fraction).max(lower_bound)));
        }
        (PropertyValue::Opacity(from), PropertyValue::Opacity(to)) => {
            value.set(PropertyValue::Opacity(between(from, to, fraction).clamp(0.0, 1.0)));
        }
        (PropertyValue::Color(from), PropertyValue::Color(to)) => {
            value.set(PropertyValue::Color(colors_between(from, to, progress)));
        }
        (PropertyValue::Shadow(from), PropertyValue::Shadow(to)) => {
            shadows_between(from, to, progress, value.empty_shadow());
        }
        (PropertyValue::Transform(from), PropertyValue::Transform(to)) => {
            blend_into(from, to, fraction, value.empty_transform());
        }
        (from, _) => value.set(from),
    }
}

/// Whether the property takes no negative values.
fn is_non_negative(property: AnimatableProperty) -> bool {
    !matches!(
        property,
        AnimatableProperty::Margin(_) | AnimatableProperty::Inset(_) | AnimatableProperty::OutlineOffset
    )
}

fn between(from: f32, to: f32, fraction: f32) -> f32 {
    from + (to - from) * fraction
}

fn lengths_between(from: ResolvedLength, to: ResolvedLength, fraction: f32, lower_bound: f32) -> ResolvedLength {
    let fraction_of_basis = match (from.fraction, to.fraction) {
        (None, None) => None,
        (from, to) => Some(between(from.unwrap_or(0.0), to.unwrap_or(0.0), fraction)),
    };
    let pixels = between(from.pixels, to.pixels, fraction);
    ResolvedLength {
        pixels: if fraction_of_basis.is_none() {
            pixels.max(lower_bound)
        } else {
            pixels
        },
        fraction: fraction_of_basis,
    }
}

/// Colors are interpolated as browsers interpolate sRGB colors: on premultiplied channels.
fn colors_between(from: Color, to: Color, progress: f64) -> Color {
    let alpha = (f64::from(from.alpha) + (f64::from(to.alpha) - f64::from(from.alpha)) * progress).clamp(0.0, 1.0);
    if alpha == 0.0 {
        return TRANSPARENT;
    }
    let channel = |from_channel: u8, to_channel: u8| {
        let premultiplied_from = f64::from(from_channel) * f64::from(from.alpha);
        let premultiplied_to = f64::from(to_channel) * f64::from(to.alpha);
        let premultiplied = premultiplied_from + (premultiplied_to - premultiplied_from) * progress;
        (premultiplied / alpha).round().clamp(0.0, 255.0) as u8
    };
    Color {
        red: channel(from.red, to.red),
        green: channel(from.green, to.green),
        blue: channel(from.blue, to.blue),
        alpha: alpha as f32,
    }
}

/// Layers are interpolated pair by pair; the shorter list is padded with transparent layers
/// that cast nothing.
fn shadows_between(from: &[ShadowLayer], to: &[ShadowLayer], progress: f64, layers: &mut Vec<ShadowLayer>) {
    let fraction = progress as f32;
    for position in 0..from.len().max(to.len()) {
        let (from_layer, to_layer) = match (from.get(position), to.get(position)) {
            (Some(from_layer), Some(to_layer)) => (*from_layer, *to_layer),
            (Some(from_layer), None) => (*from_layer, invisible_like(from_layer)),
            (None, Some(to_layer)) => (invisible_like(to_layer), *to_layer),
            (None, None) => unreachable!("the position is within the longer list"),
        };
        let pixels_between = |from: Pixels, to: Pixels| Pixels(between(from.0, to.0, fraction));
        layers.push(ShadowLayer {
            offset_x: pixels_between(from_layer.offset_x, to_layer.offset_x),
            offset_y: pixels_between(from_layer.offset_y, to_layer.offset_y),
            blur: Pixels(between(from_layer.blur.0, to_layer.blur.0, fraction).max(0.0)),
            spread: pixels_between(from_layer.spread, to_layer.spread),
            color: colors_between(from_layer.color, to_layer.color, progress),
            inset: from_layer.inset,
        });
    }
}

fn invisible_like(layer: &ShadowLayer) -> ShadowLayer {
    ShadowLayer {
        offset_x: Pixels(0.0),
        offset_y: Pixels(0.0),
        blur: Pixels(0.0),
        spread: Pixels(0.0),
        color: TRANSPARENT,
        inset: layer.inset,
    }
}
