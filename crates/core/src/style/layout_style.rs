use morok_style::{
    ContentAlignment, Display, FlexDirection, FlexWrap, GridAutoFlow, GridLine, ItemAlignment, LineStyle, Overflow,
    SelfAlignment,
};

use crate::geometry::sides::Sides;
use crate::style::declaration_source::DeclarationSource;
use crate::style::positioning::Positioning;
use crate::style::resolved_length::ResolvedLength;
use crate::style::resolved_length_or_auto::ResolvedLengthOrAuto;

/// The width of a border or an outline whose width is not declared: the CSS `medium`.
pub(crate) const MEDIUM_LINE_WIDTH: f32 = 3.0;

/// The part of a computed style that layout reads; a change here moves boxes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutStyle {
    pub display: Display,
    pub positioning: Positioning,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub width: ResolvedLengthOrAuto,
    pub height: ResolvedLengthOrAuto,
    pub min_width: ResolvedLengthOrAuto,
    pub min_height: ResolvedLengthOrAuto,
    pub max_width: ResolvedLengthOrAuto,
    pub max_height: ResolvedLengthOrAuto,
    pub margin: Sides<ResolvedLengthOrAuto>,
    pub padding: Sides<ResolvedLength>,
    pub inset: Sides<ResolvedLengthOrAuto>,
    /// The declared widths; a side without a border style takes no space, see [`Self::used_border_width`].
    pub border_width: Sides<f32>,
    pub border_style: Sides<LineStyle>,
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: ResolvedLengthOrAuto,
    pub justify_content: ContentAlignment,
    pub align_content: ContentAlignment,
    pub justify_items: ItemAlignment,
    pub align_items: ItemAlignment,
    pub justify_self: SelfAlignment,
    pub align_self: SelfAlignment,
    pub row_gap: ResolvedLength,
    pub column_gap: ResolvedLength,
    pub(crate) grid_template_columns: Option<DeclarationSource>,
    pub(crate) grid_template_rows: Option<DeclarationSource>,
    pub(crate) grid_auto_columns: Option<DeclarationSource>,
    pub(crate) grid_auto_rows: Option<DeclarationSource>,
    pub grid_auto_flow: GridAutoFlow,
    pub grid_column_start: GridLine,
    pub grid_column_end: GridLine,
    pub grid_row_start: GridLine,
    pub grid_row_end: GridLine,
}

impl LayoutStyle {
    /// The CSS initial values under the reset every surface applies: `box-sizing: border-box`,
    /// zero margin and padding, block display.
    pub(crate) const INITIAL: Self = Self {
        display: Display::Block,
        positioning: Positioning::Static,
        overflow_x: Overflow::Visible,
        overflow_y: Overflow::Visible,
        width: ResolvedLengthOrAuto::Auto,
        height: ResolvedLengthOrAuto::Auto,
        min_width: ResolvedLengthOrAuto::Auto,
        min_height: ResolvedLengthOrAuto::Auto,
        max_width: ResolvedLengthOrAuto::Auto,
        max_height: ResolvedLengthOrAuto::Auto,
        margin: Sides::uniform(ResolvedLengthOrAuto::ZERO),
        padding: Sides::uniform(ResolvedLength::ZERO),
        inset: Sides::uniform(ResolvedLengthOrAuto::Auto),
        border_width: Sides::uniform(MEDIUM_LINE_WIDTH),
        border_style: Sides::uniform(LineStyle::None),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::NoWrap,
        flex_grow: 0.0,
        flex_shrink: 1.0,
        flex_basis: ResolvedLengthOrAuto::Auto,
        justify_content: ContentAlignment::Normal,
        align_content: ContentAlignment::Normal,
        justify_items: ItemAlignment::Normal,
        align_items: ItemAlignment::Normal,
        justify_self: SelfAlignment::Auto,
        align_self: SelfAlignment::Auto,
        row_gap: ResolvedLength::ZERO,
        column_gap: ResolvedLength::ZERO,
        grid_template_columns: None,
        grid_template_rows: None,
        grid_auto_columns: None,
        grid_auto_rows: None,
        grid_auto_flow: GridAutoFlow::Row,
        grid_column_start: GridLine::Auto,
        grid_column_end: GridLine::Auto,
        grid_row_start: GridLine::Auto,
        grid_row_end: GridLine::Auto,
    };

    /// The space each border takes: its declared width, or nothing while the side has no border style.
    pub fn used_border_width(&self) -> Sides<f32> {
        let used = |width: f32, style: LineStyle| match style {
            LineStyle::None => 0.0,
            LineStyle::Solid => width,
        };
        Sides {
            top: used(self.border_width.top, self.border_style.top),
            right: used(self.border_width.right, self.border_style.right),
            bottom: used(self.border_width.bottom, self.border_style.bottom),
            left: used(self.border_width.left, self.border_style.left),
        }
    }

    /// Whether the element clips its content. Clipping the overflow of one axis clips both: CSS
    /// computes the `visible` of the other axis to `auto`.
    pub fn clips_overflow(&self) -> bool {
        self.overflow_x != Overflow::Visible || self.overflow_y != Overflow::Visible
    }

    /// Whether the content of a `Scroll` element scrolls across.
    pub fn scrolls_horizontally(&self) -> bool {
        scrolls(self.overflow_x, self.overflow_y)
    }

    /// Whether the content of a `Scroll` element scrolls down.
    pub fn scrolls_vertically(&self) -> bool {
        scrolls(self.overflow_y, self.overflow_x)
    }

    /// Whether the element generates a box.
    pub fn generates_box(&self) -> bool {
        self.display != Display::None
    }
}

/// Whether an axis of overflow `own` scrolls beside an axis of overflow `other`: `auto` and
/// `scroll` do, and so does a `visible` that CSS computes to `auto` beside a clipping axis.
fn scrolls(own: Overflow, other: Overflow) -> bool {
    match own {
        Overflow::Auto | Overflow::Scroll => true,
        Overflow::Visible => other != Overflow::Visible,
        Overflow::Hidden => false,
    }
}
