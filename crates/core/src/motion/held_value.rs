use morok_style::{Color, ShadowLayer};

use crate::motion::property_value::PropertyValue;
use crate::motion::transform_step::TransformStep;
use crate::style::resolved_length::ResolvedLength;
use crate::style::resolved_length_or_auto::ResolvedLengthOrAuto;

/// A value of an animatable property that owns its list: an end of a transition, or a value
/// worked out for a frame.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum HeldValue {
    Size(ResolvedLengthOrAuto),
    Length(ResolvedLength),
    Pixels(f32),
    Opacity(f32),
    Color(Color),
    Shadow(Vec<ShadowLayer>),
    Transform(Vec<TransformStep>),
}

impl HeldValue {
    pub(crate) fn of(value: PropertyValue<'_>) -> Self {
        let mut held = Self::Opacity(1.0);
        held.set(value);
        held
    }

    pub(crate) fn value(&self) -> PropertyValue<'_> {
        match self {
            Self::Size(size) => PropertyValue::Size(*size),
            Self::Length(length) => PropertyValue::Length(*length),
            Self::Pixels(pixels) => PropertyValue::Pixels(*pixels),
            Self::Opacity(opacity) => PropertyValue::Opacity(*opacity),
            Self::Color(color) => PropertyValue::Color(*color),
            Self::Shadow(layers) => PropertyValue::Shadow(layers),
            Self::Transform(steps) => PropertyValue::Transform(steps),
        }
    }

    /// Holds `value` from now on; a list goes into the list already held where there is one.
    pub(crate) fn set(&mut self, value: PropertyValue<'_>) {
        match (&mut *self, value) {
            (Self::Shadow(held), PropertyValue::Shadow(layers)) => {
                held.clear();
                held.extend_from_slice(layers);
            }
            (Self::Transform(held), PropertyValue::Transform(steps)) => {
                held.clear();
                held.extend_from_slice(steps);
            }
            (_, PropertyValue::Shadow(layers)) => *self = Self::Shadow(layers.to_vec()),
            (_, PropertyValue::Transform(steps)) => *self = Self::Transform(steps.to_vec()),
            (_, PropertyValue::Size(size)) => *self = Self::Size(size),
            (_, PropertyValue::Length(length)) => *self = Self::Length(length),
            (_, PropertyValue::Pixels(pixels)) => *self = Self::Pixels(pixels),
            (_, PropertyValue::Opacity(opacity)) => *self = Self::Opacity(opacity),
            (_, PropertyValue::Color(color)) => *self = Self::Color(color),
        }
    }

    /// The list to write a shadow into, emptied; a held value of another kind becomes a shadow.
    pub(crate) fn empty_shadow(&mut self) -> &mut Vec<ShadowLayer> {
        if !matches!(self, Self::Shadow(_)) {
            *self = Self::Shadow(Vec::new());
        }
        let Self::Shadow(layers) = self else {
            unreachable!("the value was made a shadow above");
        };
        layers.clear();
        layers
    }

    /// The list to write transform steps into, emptied; a held value of another kind becomes a
    /// transform.
    pub(crate) fn empty_transform(&mut self) -> &mut Vec<TransformStep> {
        if !matches!(self, Self::Transform(_)) {
            *self = Self::Transform(Vec::new());
        }
        let Self::Transform(steps) = self else {
            unreachable!("the value was made a transform above");
        };
        steps.clear();
        steps
    }
}
