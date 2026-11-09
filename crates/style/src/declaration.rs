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
use crate::paint::background_image::BackgroundImage;
use crate::paint::box_shadow::BoxShadow;
use crate::paint::color_value::ColorValue;
use crate::paint::line_style::LineStyle;
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
}
