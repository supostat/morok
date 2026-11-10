use std::borrow::Cow;

use crate::corner::Corner;
use crate::effect::backdrop_filter_function::BackdropFilterFunction;
use crate::effect::blend_mode::BlendMode;
use crate::effect::filter_function::FilterFunction;
use crate::effect::transform_function::TransformFunction;
use crate::font_families::FontFamilies;
use crate::layout::content_alignment::ContentAlignment;
use crate::layout::display::Display;
use crate::layout::flex_direction::FlexDirection;
use crate::layout::flex_wrap::FlexWrap;
use crate::layout::grid_auto_flow::GridAutoFlow;
use crate::layout::grid_line::GridLine;
use crate::layout::item_alignment::ItemAlignment;
use crate::layout::overflow::Overflow;
use crate::layout::position::Position;
use crate::layout::self_alignment::SelfAlignment;
use crate::layout::track_list_item::TrackListItem;
use crate::layout::track_size::TrackSize;
use crate::length::Length;
use crate::length_percentage::LengthPercentage;
use crate::length_percentage_auto::LengthPercentageAuto;
use crate::motion::animatable_properties::AnimatableProperties;
use crate::motion::animatable_property::AnimatableProperty;
use crate::motion::animation_direction::AnimationDirection;
use crate::motion::animation_name::AnimationName;
use crate::motion::easing_value::EasingValue;
use crate::motion::fill_mode::FillMode;
use crate::motion::iteration_count::IterationCount;
use crate::motion::time_value::TimeValue;
use crate::paint::background_image::BackgroundImage;
use crate::paint::box_shadow::BoxShadow;
use crate::paint::color_value::ColorValue;
use crate::paint::line_style::LineStyle;
use crate::paint::object_fit::ObjectFit;
use crate::side::Side;
use crate::text::font_style::FontStyle;
use crate::text::font_variant_numeric::FontVariantNumeric;
use crate::text::font_weight_value::FontWeightValue;
use crate::text::line_height::LineHeight;
use crate::text::text_align::TextAlign;
use crate::text::text_decoration::TextDecoration;
use crate::text::text_overflow::TextOverflow;
use crate::text::white_space::WhiteSpace;
use crate::token::Token;

/// One longhand property with its value; a shorthand compiles into the declarations of its longhands.
#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    Display(Display),
    Position(Position),
    OverflowX(Overflow),
    OverflowY(Overflow),
    Width(LengthPercentageAuto),
    Height(LengthPercentageAuto),
    MinWidth(LengthPercentageAuto),
    MinHeight(LengthPercentageAuto),
    MaxWidth(LengthPercentageAuto),
    MaxHeight(LengthPercentageAuto),
    Margin(Side, LengthPercentageAuto),
    Padding(Side, LengthPercentage),
    Inset(Side, LengthPercentageAuto),
    FlexDirection(FlexDirection),
    FlexWrap(FlexWrap),
    FlexGrow(f32),
    FlexShrink(f32),
    FlexBasis(LengthPercentageAuto),
    JustifyContent(ContentAlignment),
    AlignContent(ContentAlignment),
    JustifyItems(ItemAlignment),
    AlignItems(ItemAlignment),
    JustifySelf(SelfAlignment),
    AlignSelf(SelfAlignment),
    RowGap(LengthPercentage),
    ColumnGap(LengthPercentage),
    GridTemplateColumns(Cow<'static, [TrackListItem]>),
    GridTemplateRows(Cow<'static, [TrackListItem]>),
    GridAutoColumns(Cow<'static, [TrackSize]>),
    GridAutoRows(Cow<'static, [TrackSize]>),
    GridAutoFlow(GridAutoFlow),
    GridColumnStart(GridLine),
    GridColumnEnd(GridLine),
    GridRowStart(GridLine),
    GridRowEnd(GridLine),
    BackgroundColor(ColorValue),
    BackgroundImage(BackgroundImage),
    BorderWidth(Side, Length),
    BorderStyle(Side, LineStyle),
    BorderColor(Side, ColorValue),
    BorderRadius(Corner, Length),
    BoxShadow(BoxShadow),
    OutlineWidth(Length),
    OutlineStyle(LineStyle),
    OutlineColor(ColorValue),
    OutlineOffset(Length),
    Color(ColorValue),
    FontFamily(Token<FontFamilies>),
    FontSize(Length),
    FontWeight(FontWeightValue),
    FontStyle(FontStyle),
    LineHeight(LineHeight),
    /// The space added after every character; `normal` is no space.
    LetterSpacing(Length),
    TextAlign(TextAlign),
    TextDecoration(TextDecoration),
    TextOverflow(TextOverflow),
    WhiteSpace(WhiteSpace),
    FontVariantNumeric(FontVariantNumeric),
    /// From `0.0`, invisible, to `1.0`, opaque; the element and its content fade as a group.
    Opacity(f32),
    /// `none` is an empty list.
    Transform(Cow<'static, [TransformFunction]>),
    /// The point of the border box the transform functions turn and scale about.
    TransformOrigin(LengthPercentage, LengthPercentage),
    /// `none` is an empty list.
    Filter(Cow<'static, [FilterFunction]>),
    /// `none` is an empty list.
    BackdropFilter(Cow<'static, [BackdropFilterFunction]>),
    MixBlendMode(BlendMode),
    ObjectFit(ObjectFit),
    /// The properties of each transition; an empty entry, `none`, transitions nothing.
    TransitionProperty(Cow<'static, [AnimatableProperties]>),
    TransitionDuration(Cow<'static, [TimeValue]>),
    TransitionTimingFunction(Cow<'static, [EasingValue]>),
    TransitionDelay(Cow<'static, [TimeValue]>),
    AnimationName(Cow<'static, [AnimationName]>),
    AnimationDuration(Cow<'static, [TimeValue]>),
    /// The easing of every interval between two keyframes.
    AnimationTimingFunction(Cow<'static, [EasingValue]>),
    AnimationDelay(Cow<'static, [TimeValue]>),
    AnimationIterationCount(Cow<'static, [IterationCount]>),
    AnimationDirection(Cow<'static, [AnimationDirection]>),
    AnimationFillMode(Cow<'static, [FillMode]>),
}

impl Declaration {
    /// The property a transition or a keyframe animation interpolates by this declaration;
    /// none for a property that is not animatable.
    pub fn animatable_property(&self) -> Option<AnimatableProperty> {
        Some(match self {
            Self::Width(_) => AnimatableProperty::Width,
            Self::Height(_) => AnimatableProperty::Height,
            Self::MinWidth(_) => AnimatableProperty::MinWidth,
            Self::MinHeight(_) => AnimatableProperty::MinHeight,
            Self::MaxWidth(_) => AnimatableProperty::MaxWidth,
            Self::MaxHeight(_) => AnimatableProperty::MaxHeight,
            Self::Margin(side, _) => AnimatableProperty::Margin(*side),
            Self::Padding(side, _) => AnimatableProperty::Padding(*side),
            Self::Inset(side, _) => AnimatableProperty::Inset(*side),
            Self::RowGap(_) => AnimatableProperty::RowGap,
            Self::ColumnGap(_) => AnimatableProperty::ColumnGap,
            Self::BorderWidth(side, _) => AnimatableProperty::BorderWidth(*side),
            Self::BorderColor(side, _) => AnimatableProperty::BorderColor(*side),
            Self::BorderRadius(corner, _) => AnimatableProperty::BorderRadius(*corner),
            Self::OutlineWidth(_) => AnimatableProperty::OutlineWidth,
            Self::OutlineColor(_) => AnimatableProperty::OutlineColor,
            Self::OutlineOffset(_) => AnimatableProperty::OutlineOffset,
            Self::Color(_) => AnimatableProperty::Color,
            Self::BackgroundColor(_) => AnimatableProperty::BackgroundColor,
            Self::BoxShadow(_) => AnimatableProperty::BoxShadow,
            Self::Opacity(_) => AnimatableProperty::Opacity,
            Self::Transform(_) => AnimatableProperty::Transform,
            _ => return None,
        })
    }
}
