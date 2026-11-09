use morok_style::{Declaration, LengthPercentage, StyleSheet, TrackBreadth, TrackListItem, TrackSize};

use crate::layout::track_bound::TrackBound;
use crate::layout::track_list::TrackList;
use crate::layout::track_slot::TrackSlot;
use crate::style::declaration_source::DeclarationSource;
use crate::style::inline_style::InlineStyle;
use crate::style::layout_style::LayoutStyle;

/// The grid track lists of one element, read in place from the declarations that won the cascade.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GridTracks<'a> {
    pub(crate) style: &'a LayoutStyle,
    pub(crate) style_sheet: &'a StyleSheet,
    pub(crate) inline_style: &'a InlineStyle,
}

impl<'a> GridTracks<'a> {
    pub(crate) fn template_columns(self) -> Option<&'a [TrackListItem]> {
        self.style
            .grid_template_columns
            .map(|source| match self.declaration(source) {
                Declaration::GridTemplateColumns(tracks) => &**tracks,
                other => unreachable!("the source of grid-template-columns names {other:?}"),
            })
    }

    pub(crate) fn template_rows(self) -> Option<&'a [TrackListItem]> {
        self.style
            .grid_template_rows
            .map(|source| match self.declaration(source) {
                Declaration::GridTemplateRows(tracks) => &**tracks,
                other => unreachable!("the source of grid-template-rows names {other:?}"),
            })
    }

    /// Empty while the property is not declared: implicit columns are then sized `auto`.
    pub(crate) fn auto_columns(self) -> &'a [TrackSize] {
        self.style
            .grid_auto_columns
            .map_or(&[], |source| match self.declaration(source) {
                Declaration::GridAutoColumns(tracks) => &**tracks,
                other => unreachable!("the source of grid-auto-columns names {other:?}"),
            })
    }

    /// Empty while the property is not declared: implicit rows are then sized `auto`.
    pub(crate) fn auto_rows(self) -> &'a [TrackSize] {
        self.style
            .grid_auto_rows
            .map_or(&[], |source| match self.declaration(source) {
                Declaration::GridAutoRows(tracks) => &**tracks,
                other => unreachable!("the source of grid-auto-rows names {other:?}"),
            })
    }

    /// The length a slot names. A slot is made only for a length that exists, so a slot that
    /// names anything else is a defect of the layout pass.
    pub(crate) fn length(self, slot: TrackSlot) -> &'a LengthPercentage {
        let track = self.track(slot);
        match (track, slot.bound) {
            (TrackSize::Breadth(TrackBreadth::LengthPercentage(length)), TrackBound::Minimum | TrackBound::Maximum)
            | (
                TrackSize::MinMax {
                    min: TrackBreadth::LengthPercentage(length),
                    ..
                },
                TrackBound::Minimum,
            )
            | (
                TrackSize::MinMax {
                    max: TrackBreadth::LengthPercentage(length),
                    ..
                },
                TrackBound::Maximum,
            )
            | (TrackSize::FitContent(length), TrackBound::FitContentLimit) => length,
            _ => unreachable!("{slot:?} names no length of {track:?}"),
        }
    }

    fn track(self, slot: TrackSlot) -> &'a TrackSize {
        let template = match slot.list {
            TrackList::TemplateColumns => self.template_columns(),
            TrackList::TemplateRows => self.template_rows(),
            TrackList::AutoColumns => return &self.auto_columns()[slot.entry],
            TrackList::AutoRows => return &self.auto_rows()[slot.entry],
        };
        let template = template.unwrap_or_else(|| unreachable!("{slot:?} names a list that is not declared"));
        match &template[slot.entry] {
            TrackListItem::Track(track) => track,
            TrackListItem::Repeat { tracks, .. } => &tracks[slot.repeated],
        }
    }

    fn declaration(self, source: DeclarationSource) -> &'a Declaration {
        source.declaration(self.style_sheet, self.inline_style)
    }
}
