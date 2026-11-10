use std::iter;

use morok_style::{ContentAlignment, Display, GridLine, ItemAlignment, SelfAlignment, Side, Theme};
use taffy::style_helpers::{TaffyAuto, TaffyGridLine};
use taffy::{
    AlignContent, AlignItems, AlignSelf, BlockContainerStyle, BlockItemStyle, BoxGenerationMode, CoreStyle, Dimension,
    FlexDirection, FlexWrap, FlexboxContainerStyle, FlexboxItemStyle, GridAutoFlow, GridContainerStyle, GridItemStyle,
    GridPlacement, GridTemplateArea, JustifyContent, LengthPercentage, LengthPercentageAuto, Line, OofItemStyle,
    Overflow, Point, Position, Rect, Size,
};

use crate::geometry::sides::Sides;
use crate::layout::calc_handle::CalcHandle;
use crate::layout::grid_tracks::GridTracks;
use crate::layout::length_slot::LengthSlot;
use crate::layout::taffy_length::TaffyLength;
use crate::layout::template_tracks::TemplateTracks;
use crate::layout::track_list::TrackList;
use crate::layout::track_repetition::{LineNames, TrackRepetition};
use crate::layout::track_sizes::TrackSizes;
use crate::layout::track_sizing::TrackSizing;
use crate::style::layout_style::LayoutStyle;
use crate::style::positioning::Positioning;
use crate::style::resolved_length::ResolvedLength;
use crate::style::resolved_length_or_auto::ResolvedLengthOrAuto;

/// The layout style of one element in the vocabulary of Taffy. Values are converted when Taffy
/// asks for them: no Taffy style is stored.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TaffyStyle<'a> {
    /// The slot of the element in its arena; names the element in `calc()` handles.
    pub(crate) element_index: u32,
    pub(crate) style: &'a LayoutStyle,
    pub(crate) tracks: GridTracks<'a>,
    pub(crate) theme: &'a Theme,
    /// Whether the element is the root: the box of the root is the viewport unless it is sized.
    pub(crate) fills_viewport: bool,
}

impl<'a> TaffyStyle<'a> {
    fn length<T: TaffyLength>(&self, length: ResolvedLength, slot: LengthSlot) -> T {
        let place = CalcHandle {
            element_index: self.element_index,
            slot,
        };
        T::of(length, place)
    }

    fn length_or_auto<T: TaffyLength + TaffyAuto>(&self, length: ResolvedLengthOrAuto, slot: LengthSlot) -> T {
        match length {
            ResolvedLengthOrAuto::Auto => T::AUTO,
            ResolvedLengthOrAuto::Length(length) => self.length(length, slot),
        }
    }

    /// A width or a height. Left `auto`, that of the root is the whole of the viewport: the root
    /// is the surface, and its background is what the surface shows behind everything else.
    fn dimension(&self, length: ResolvedLengthOrAuto, slot: LengthSlot) -> Dimension {
        match length {
            ResolvedLengthOrAuto::Auto if self.fills_viewport => Dimension::percent(1.0),
            length => self.length_or_auto(length, slot),
        }
    }

    fn sides_or_auto(
        &self,
        sides: &Sides<ResolvedLengthOrAuto>,
        slot: fn(Side) -> LengthSlot,
    ) -> Rect<LengthPercentageAuto> {
        Rect {
            left: self.length_or_auto(sides.left, slot(Side::Left)),
            right: self.length_or_auto(sides.right, slot(Side::Right)),
            top: self.length_or_auto(sides.top, slot(Side::Top)),
            bottom: self.length_or_auto(sides.bottom, slot(Side::Bottom)),
        }
    }

    fn gap(&self) -> Size<LengthPercentage> {
        Size {
            width: self.length(self.style.column_gap, LengthSlot::ColumnGap),
            height: self.length(self.style.row_gap, LengthSlot::RowGap),
        }
    }

    fn track_sizing(&self, list: TrackList) -> TrackSizing<'a> {
        TrackSizing {
            theme: self.theme,
            element_index: self.element_index,
            list,
        }
    }

    fn grid_row(&self) -> Line<GridPlacement<String>> {
        Line {
            start: grid_placement(self.style.grid_row_start),
            end: grid_placement(self.style.grid_row_end),
        }
    }

    fn grid_column(&self) -> Line<GridPlacement<String>> {
        Line {
            start: grid_placement(self.style.grid_column_start),
            end: grid_placement(self.style.grid_column_end),
        }
    }
}

fn grid_placement(line: GridLine) -> GridPlacement<String> {
    match line {
        GridLine::Auto => GridPlacement::Auto,
        GridLine::Line(number) => GridPlacement::from_line_index(number),
        GridLine::Span(tracks) => GridPlacement::Span(tracks),
    }
}

fn content_alignment(alignment: ContentAlignment) -> AlignContent {
    match alignment {
        ContentAlignment::Normal => AlignContent::NORMAL,
        ContentAlignment::Start => AlignContent::START,
        ContentAlignment::End => AlignContent::END,
        ContentAlignment::FlexStart => AlignContent::FLEX_START,
        ContentAlignment::FlexEnd => AlignContent::FLEX_END,
        ContentAlignment::Center => AlignContent::CENTER,
        ContentAlignment::Stretch => AlignContent::STRETCH,
        ContentAlignment::SpaceBetween => AlignContent::SPACE_BETWEEN,
        ContentAlignment::SpaceAround => AlignContent::SPACE_AROUND,
        ContentAlignment::SpaceEvenly => AlignContent::SPACE_EVENLY,
    }
}

fn item_alignment(alignment: ItemAlignment) -> AlignItems {
    match alignment {
        ItemAlignment::Normal => AlignItems::NORMAL,
        ItemAlignment::Start => AlignItems::START,
        ItemAlignment::End => AlignItems::END,
        ItemAlignment::FlexStart => AlignItems::FLEX_START,
        ItemAlignment::FlexEnd => AlignItems::FLEX_END,
        ItemAlignment::Center => AlignItems::CENTER,
        ItemAlignment::Baseline => AlignItems::BASELINE,
        ItemAlignment::Stretch => AlignItems::STRETCH,
    }
}

fn self_alignment(alignment: SelfAlignment) -> Option<AlignSelf> {
    match alignment {
        SelfAlignment::Auto => None,
        SelfAlignment::Item(alignment) => Some(item_alignment(alignment)),
    }
}

impl CoreStyle for TaffyStyle<'_> {
    type CustomIdent = String;

    fn box_generation_mode(&self) -> BoxGenerationMode {
        if self.style.generates_box() {
            BoxGenerationMode::Normal
        } else {
            BoxGenerationMode::None
        }
    }

    fn is_block(&self) -> bool {
        self.style.display == Display::Block
    }

    fn overflow(&self) -> Point<Overflow> {
        let axis = |scrolls: bool| {
            if scrolls {
                Overflow::Scroll
            } else if self.style.clips_overflow() {
                Overflow::Hidden
            } else {
                Overflow::Visible
            }
        };
        Point {
            x: axis(self.style.scrolls_horizontally()),
            y: axis(self.style.scrolls_vertically()),
        }
    }

    fn position(&self) -> Position {
        match self.style.positioning {
            Positioning::Static => Position::Static,
            Positioning::Relative | Positioning::Sticky => Position::Relative,
            Positioning::Absolute => Position::Absolute,
        }
    }

    /// The insets of a sticky element do not move its box: they bound the shift a scroll gives it.
    fn inset(&self) -> Rect<LengthPercentageAuto> {
        if self.style.positioning == Positioning::Sticky {
            return Rect {
                left: LengthPercentageAuto::auto(),
                right: LengthPercentageAuto::auto(),
                top: LengthPercentageAuto::auto(),
                bottom: LengthPercentageAuto::auto(),
            };
        }
        self.sides_or_auto(&self.style.inset, LengthSlot::Inset)
    }

    fn size(&self) -> Size<Dimension> {
        Size {
            width: self.dimension(self.style.width, LengthSlot::Width),
            height: self.dimension(self.style.height, LengthSlot::Height),
        }
    }

    fn min_size(&self) -> Size<LengthPercentageAuto> {
        Size {
            width: self.length_or_auto(self.style.min_width, LengthSlot::MinWidth),
            height: self.length_or_auto(self.style.min_height, LengthSlot::MinHeight),
        }
    }

    fn max_size(&self) -> Size<LengthPercentageAuto> {
        Size {
            width: self.length_or_auto(self.style.max_width, LengthSlot::MaxWidth),
            height: self.length_or_auto(self.style.max_height, LengthSlot::MaxHeight),
        }
    }

    fn margin(&self) -> Rect<LengthPercentageAuto> {
        self.sides_or_auto(&self.style.margin, LengthSlot::Margin)
    }

    fn padding(&self) -> Rect<LengthPercentage> {
        let padding = &self.style.padding;
        Rect {
            left: self.length(padding.left, LengthSlot::Padding(Side::Left)),
            right: self.length(padding.right, LengthSlot::Padding(Side::Right)),
            top: self.length(padding.top, LengthSlot::Padding(Side::Top)),
            bottom: self.length(padding.bottom, LengthSlot::Padding(Side::Bottom)),
        }
    }

    fn border(&self) -> Rect<LengthPercentage> {
        let border = self.style.used_border_width();
        Rect {
            left: LengthPercentage::length(border.left),
            right: LengthPercentage::length(border.right),
            top: LengthPercentage::length(border.top),
            bottom: LengthPercentage::length(border.bottom),
        }
    }
}

impl OofItemStyle for TaffyStyle<'_> {
    fn align_self(&self) -> Option<AlignSelf> {
        self_alignment(self.style.align_self)
    }

    fn justify_self(&self) -> Option<AlignSelf> {
        self_alignment(self.style.justify_self)
    }

    fn grid_row(&self) -> Line<GridPlacement<String>> {
        TaffyStyle::grid_row(self)
    }

    fn grid_column(&self) -> Line<GridPlacement<String>> {
        TaffyStyle::grid_column(self)
    }
}

impl FlexboxContainerStyle for TaffyStyle<'_> {
    fn flex_direction(&self) -> FlexDirection {
        match self.style.flex_direction {
            morok_style::FlexDirection::Row => FlexDirection::Row,
            morok_style::FlexDirection::RowReverse => FlexDirection::RowReverse,
            morok_style::FlexDirection::Column => FlexDirection::Column,
            morok_style::FlexDirection::ColumnReverse => FlexDirection::ColumnReverse,
        }
    }

    fn flex_wrap(&self) -> FlexWrap {
        match self.style.flex_wrap {
            morok_style::FlexWrap::NoWrap => FlexWrap::NoWrap,
            morok_style::FlexWrap::Wrap => FlexWrap::Wrap,
            morok_style::FlexWrap::WrapReverse => FlexWrap::WrapReverse,
        }
    }

    fn gap(&self) -> Size<LengthPercentage> {
        TaffyStyle::gap(self)
    }

    fn align_content(&self) -> AlignContent {
        content_alignment(self.style.align_content)
    }

    fn align_items(&self) -> AlignItems {
        item_alignment(self.style.align_items)
    }

    fn justify_content(&self) -> JustifyContent {
        content_alignment(self.style.justify_content)
    }
}

impl FlexboxItemStyle for TaffyStyle<'_> {
    fn flex_basis(&self) -> Dimension {
        self.length_or_auto(self.style.flex_basis, LengthSlot::FlexBasis)
    }

    fn flex_grow(&self) -> f32 {
        self.style.flex_grow
    }

    fn flex_shrink(&self) -> f32 {
        self.style.flex_shrink
    }

    fn align_self(&self) -> Option<AlignSelf> {
        self_alignment(self.style.align_self)
    }
}

impl GridContainerStyle for TaffyStyle<'_> {
    type Repetition<'a>
        = TrackRepetition<'a>
    where
        Self: 'a;

    type TemplateTrackList<'a>
        = TemplateTracks<'a>
    where
        Self: 'a;

    type AutoTrackList<'a>
        = TrackSizes<'a>
    where
        Self: 'a;

    type TemplateLineNames<'a>
        = LineNames<'a>
    where
        Self: 'a;

    type GridTemplateAreas<'a>
        = iter::Empty<GridTemplateArea<String>>
    where
        Self: 'a;

    fn grid_template_rows(&self) -> Option<Self::TemplateTrackList<'_>> {
        let entries = self.tracks.template_rows()?;
        Some(TemplateTracks::new(entries, self.track_sizing(TrackList::TemplateRows)))
    }

    fn grid_template_columns(&self) -> Option<Self::TemplateTrackList<'_>> {
        let entries = self.tracks.template_columns()?;
        Some(TemplateTracks::new(
            entries,
            self.track_sizing(TrackList::TemplateColumns),
        ))
    }

    fn grid_auto_rows(&self) -> Self::AutoTrackList<'_> {
        TrackSizes::entries(self.tracks.auto_rows(), self.track_sizing(TrackList::AutoRows))
    }

    fn grid_auto_columns(&self) -> Self::AutoTrackList<'_> {
        TrackSizes::entries(self.tracks.auto_columns(), self.track_sizing(TrackList::AutoColumns))
    }

    fn grid_template_areas(&self) -> Option<Self::GridTemplateAreas<'_>> {
        None
    }

    fn grid_template_column_names(&self) -> Option<Self::TemplateLineNames<'_>> {
        None
    }

    fn grid_template_row_names(&self) -> Option<Self::TemplateLineNames<'_>> {
        None
    }

    fn grid_auto_flow(&self) -> GridAutoFlow {
        match self.style.grid_auto_flow {
            morok_style::GridAutoFlow::Row => GridAutoFlow::Row,
            morok_style::GridAutoFlow::Column => GridAutoFlow::Column,
            morok_style::GridAutoFlow::RowDense => GridAutoFlow::RowDense,
            morok_style::GridAutoFlow::ColumnDense => GridAutoFlow::ColumnDense,
        }
    }

    fn gap(&self) -> Size<LengthPercentage> {
        TaffyStyle::gap(self)
    }

    fn align_content(&self) -> AlignContent {
        content_alignment(self.style.align_content)
    }

    fn justify_content(&self) -> JustifyContent {
        content_alignment(self.style.justify_content)
    }

    fn align_items(&self) -> AlignItems {
        item_alignment(self.style.align_items)
    }

    fn justify_items(&self) -> AlignItems {
        item_alignment(self.style.justify_items)
    }
}

impl GridItemStyle for TaffyStyle<'_> {
    fn grid_row(&self) -> Line<GridPlacement<String>> {
        TaffyStyle::grid_row(self)
    }

    fn grid_column(&self) -> Line<GridPlacement<String>> {
        TaffyStyle::grid_column(self)
    }

    fn align_self(&self) -> Option<AlignSelf> {
        self_alignment(self.style.align_self)
    }

    fn justify_self(&self) -> Option<AlignSelf> {
        self_alignment(self.style.justify_self)
    }
}

impl BlockContainerStyle for TaffyStyle<'_> {
    fn align_content(&self) -> AlignContent {
        content_alignment(self.style.align_content)
    }

    fn justify_items(&self) -> AlignItems {
        item_alignment(self.style.justify_items)
    }
}

impl BlockItemStyle for TaffyStyle<'_> {
    fn align_self(&self) -> Option<AlignSelf> {
        self_alignment(self.style.align_self)
    }

    fn justify_self(&self) -> Option<AlignSelf> {
        self_alignment(self.style.justify_self)
    }

    fn align_content(&self) -> AlignContent {
        content_alignment(self.style.align_content)
    }
}
