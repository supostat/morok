use morok_style::{AnimatableProperty, Color, ShadowLayer};

use crate::motion::transform_step::TransformStep;
use crate::style::computed_style::ComputedStyle;
use crate::style::resolved_length::ResolvedLength;
use crate::style::resolved_length_or_auto::ResolvedLengthOrAuto;
use crate::style::style_lists::StyleLists;

/// The value of an animatable property as a style holds it; a list is borrowed from where it lives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PropertyValue<'a> {
    /// A size, a margin or an inset.
    Size(ResolvedLengthOrAuto),
    /// A padding or a gap.
    Length(ResolvedLength),
    /// A border width, a corner radius, the width or the offset of the outline.
    Pixels(f32),
    Opacity(f32),
    Color(Color),
    Shadow(&'a [ShadowLayer]),
    Transform(&'a [TransformStep]),
}

impl<'a> PropertyValue<'a> {
    /// The value of the property in a style and the lists beside it.
    pub(crate) fn read(property: AnimatableProperty, style: &ComputedStyle, lists: &'a StyleLists) -> Self {
        let layout = &style.layout;
        let paint = &style.paint;
        match property {
            AnimatableProperty::Width => Self::Size(layout.width),
            AnimatableProperty::Height => Self::Size(layout.height),
            AnimatableProperty::MinWidth => Self::Size(layout.min_width),
            AnimatableProperty::MinHeight => Self::Size(layout.min_height),
            AnimatableProperty::MaxWidth => Self::Size(layout.max_width),
            AnimatableProperty::MaxHeight => Self::Size(layout.max_height),
            AnimatableProperty::Margin(side) => Self::Size(layout.margin[side]),
            AnimatableProperty::Inset(side) => Self::Size(layout.inset[side]),
            AnimatableProperty::Padding(side) => Self::Length(layout.padding[side]),
            AnimatableProperty::RowGap => Self::Length(layout.row_gap),
            AnimatableProperty::ColumnGap => Self::Length(layout.column_gap),
            AnimatableProperty::BorderWidth(side) => Self::Pixels(layout.border_width[side]),
            AnimatableProperty::BorderRadius(corner) => Self::Pixels(paint.border_radius[corner]),
            AnimatableProperty::OutlineWidth => Self::Pixels(paint.outline_width),
            AnimatableProperty::OutlineOffset => Self::Pixels(paint.outline_offset),
            AnimatableProperty::BorderColor(side) => Self::Color(paint.border_color[side]),
            AnimatableProperty::OutlineColor => Self::Color(paint.outline_color),
            AnimatableProperty::Color => Self::Color(paint.color),
            AnimatableProperty::BackgroundColor => Self::Color(paint.background_color),
            AnimatableProperty::Opacity => Self::Opacity(style.effects.opacity),
            AnimatableProperty::BoxShadow => Self::Shadow(&lists.box_shadow),
            AnimatableProperty::Transform => Self::Transform(&lists.transform),
        }
    }

    /// Sets the property of a style and the lists beside it to this value.
    ///
    /// # Panics
    ///
    /// When the value is not of the kind the property takes.
    pub(crate) fn write(self, property: AnimatableProperty, style: &mut ComputedStyle, lists: &mut StyleLists) {
        let layout = &mut style.layout;
        let paint = &mut style.paint;
        match (property, self) {
            (AnimatableProperty::Width, Self::Size(size)) => layout.width = size,
            (AnimatableProperty::Height, Self::Size(size)) => layout.height = size,
            (AnimatableProperty::MinWidth, Self::Size(size)) => layout.min_width = size,
            (AnimatableProperty::MinHeight, Self::Size(size)) => layout.min_height = size,
            (AnimatableProperty::MaxWidth, Self::Size(size)) => layout.max_width = size,
            (AnimatableProperty::MaxHeight, Self::Size(size)) => layout.max_height = size,
            (AnimatableProperty::Margin(side), Self::Size(margin)) => layout.margin[side] = margin,
            (AnimatableProperty::Inset(side), Self::Size(inset)) => layout.inset[side] = inset,
            (AnimatableProperty::Padding(side), Self::Length(length)) => layout.padding[side] = length,
            (AnimatableProperty::RowGap, Self::Length(length)) => layout.row_gap = length,
            (AnimatableProperty::ColumnGap, Self::Length(length)) => layout.column_gap = length,
            (AnimatableProperty::BorderWidth(side), Self::Pixels(pixels)) => layout.border_width[side] = pixels,
            (AnimatableProperty::BorderRadius(corner), Self::Pixels(pixels)) => paint.border_radius[corner] = pixels,
            (AnimatableProperty::OutlineWidth, Self::Pixels(pixels)) => paint.outline_width = pixels,
            (AnimatableProperty::OutlineOffset, Self::Pixels(pixels)) => paint.outline_offset = pixels,
            (AnimatableProperty::BorderColor(side), Self::Color(color)) => paint.border_color[side] = color,
            (AnimatableProperty::OutlineColor, Self::Color(color)) => paint.outline_color = color,
            (AnimatableProperty::Color, Self::Color(color)) => paint.color = color,
            (AnimatableProperty::BackgroundColor, Self::Color(color)) => paint.background_color = color,
            (AnimatableProperty::Opacity, Self::Opacity(opacity)) => style.effects.opacity = opacity,
            (AnimatableProperty::BoxShadow, Self::Shadow(layers)) => {
                lists.box_shadow.clear();
                lists.box_shadow.extend_from_slice(layers);
            }
            (AnimatableProperty::Transform, Self::Transform(steps)) => {
                lists.transform.clear();
                lists.transform.extend_from_slice(steps);
                style.effects.has_transform = !steps.is_empty();
            }
            (property, value) => panic!("{property:?} takes no {value:?}"),
        }
    }
}
