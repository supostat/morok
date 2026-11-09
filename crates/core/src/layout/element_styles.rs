use morok_style::{StyleSheet, Theme};

use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::layout::calc_handle::CalcHandle;
use crate::layout::grid_tracks::GridTracks;
use crate::layout::length_slot::LengthSlot;
use crate::layout::taffy_style::TaffyStyle;
use crate::style::layout_style::LayoutStyle;
use crate::style::resolved_length::ResolvedLength;
use crate::style::resolved_length_or_auto::ResolvedLengthOrAuto;

/// The layout styles of the elements of a tree, as Taffy reads them.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ElementStyles<'a> {
    pub(crate) elements: &'a ElementArena,
    pub(crate) style_sheet: &'a StyleSheet,
    pub(crate) theme: &'a Theme,
}

impl<'a> ElementStyles<'a> {
    pub(crate) fn taffy_style(self, id: ElementId) -> TaffyStyle<'a> {
        let element = &self.elements[id];
        TaffyStyle {
            element_index: id.index(),
            style: &element.style.layout,
            tracks: self.grid_tracks(id),
            theme: self.theme,
            fills_viewport: element.parent.is_none(),
        }
    }

    /// The value of the length a `calc()` handle names, against the basis Taffy found for it.
    pub(crate) fn calc_value(self, handle: *const (), basis: f32) -> f32 {
        let handle = CalcHandle::from_pointer(handle);
        let id = self.elements.id_at(handle.element_index);
        let length = match handle.slot {
            LengthSlot::Track(slot) => ResolvedLength::of(self.grid_tracks(id).length(slot), self.theme),
            plain => plain_length(&self.elements[id].style.layout, plain),
        };
        length.against(basis)
    }

    fn grid_tracks(self, id: ElementId) -> GridTracks<'a> {
        let element = &self.elements[id];
        GridTracks {
            style: &element.style.layout,
            style_sheet: self.style_sheet,
            inline_style: &element.inline_style,
        }
    }
}

/// A handle is made only for a length that mixes pixels with a percentage, so a slot that holds
/// `auto` or a track is a defect of the layout pass.
fn plain_length(style: &LayoutStyle, slot: LengthSlot) -> ResolvedLength {
    let length = match slot {
        LengthSlot::Width => style.width,
        LengthSlot::Height => style.height,
        LengthSlot::MinWidth => style.min_width,
        LengthSlot::MinHeight => style.min_height,
        LengthSlot::MaxWidth => style.max_width,
        LengthSlot::MaxHeight => style.max_height,
        LengthSlot::Margin(side) => style.margin[side],
        LengthSlot::Inset(side) => style.inset[side],
        LengthSlot::FlexBasis => style.flex_basis,
        LengthSlot::Padding(side) => return style.padding[side],
        LengthSlot::RowGap => return style.row_gap,
        LengthSlot::ColumnGap => return style.column_gap,
        LengthSlot::Track(_) => unreachable!("{slot:?} is not a plain slot"),
    };
    match length {
        ResolvedLengthOrAuto::Length(length) => length,
        ResolvedLengthOrAuto::Auto => unreachable!("{slot:?} holds no length"),
    }
}
