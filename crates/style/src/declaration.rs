use std::borrow::Cow;

use crate::corner::Corner;
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
}
