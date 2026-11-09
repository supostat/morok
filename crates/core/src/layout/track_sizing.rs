use morok_style::{LengthPercentage, Theme, TrackBreadth, TrackSize};
use taffy::{MaxTrackSizingFunction, MinMax, MinTrackSizingFunction, TrackSizingFunction};

use crate::layout::calc_handle::CalcHandle;
use crate::layout::length_slot::LengthSlot;
use crate::layout::taffy_length::TaffyLength;
use crate::layout::track_bound::TrackBound;
use crate::layout::track_list::TrackList;
use crate::layout::track_slot::TrackSlot;
use crate::style::resolved_length::ResolvedLength;

/// Turns the track sizes of one list of one element into the sizing functions Taffy lays out.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TrackSizing<'a> {
    pub(crate) theme: &'a Theme,
    pub(crate) element_index: u32,
    pub(crate) list: TrackList,
}

impl TrackSizing<'_> {
    pub(crate) fn sizing_function(self, track: &TrackSize, entry: usize, repeated: usize) -> TrackSizingFunction {
        let place = |bound| CalcHandle {
            element_index: self.element_index,
            slot: LengthSlot::Track(TrackSlot {
                list: self.list,
                entry,
                repeated,
                bound,
            }),
        };
        match track {
            TrackSize::Breadth(breadth) => MinMax {
                min: self.minimum(breadth, place(TrackBound::Minimum)),
                max: self.maximum(breadth, place(TrackBound::Maximum)),
            },
            TrackSize::MinMax { min, max } => MinMax {
                min: self.minimum(min, place(TrackBound::Minimum)),
                max: self.maximum(max, place(TrackBound::Maximum)),
            },
            TrackSize::FitContent(limit) => MinMax {
                min: MinTrackSizingFunction::auto(),
                max: self.fit_content(limit, place(TrackBound::FitContentLimit)),
            },
        }
    }

    /// A flexible track has an automatic minimum, as in CSS.
    fn minimum(self, breadth: &TrackBreadth, place: CalcHandle) -> MinTrackSizingFunction {
        match breadth {
            TrackBreadth::LengthPercentage(length) => TaffyLength::of(ResolvedLength::of(length, self.theme), place),
            TrackBreadth::Flex(_) | TrackBreadth::Auto => MinTrackSizingFunction::auto(),
            TrackBreadth::MinContent => MinTrackSizingFunction::min_content(),
            TrackBreadth::MaxContent => MinTrackSizingFunction::max_content(),
        }
    }

    fn maximum(self, breadth: &TrackBreadth, place: CalcHandle) -> MaxTrackSizingFunction {
        match breadth {
            TrackBreadth::LengthPercentage(length) => TaffyLength::of(ResolvedLength::of(length, self.theme), place),
            TrackBreadth::Flex(share) => MaxTrackSizingFunction::fr(*share),
            TrackBreadth::Auto => MaxTrackSizingFunction::auto(),
            TrackBreadth::MinContent => MaxTrackSizingFunction::min_content(),
            TrackBreadth::MaxContent => MaxTrackSizingFunction::max_content(),
        }
    }

    fn fit_content(self, limit: &LengthPercentage, place: CalcHandle) -> MaxTrackSizingFunction {
        let limit = ResolvedLength::of(limit, self.theme);
        match limit.fraction {
            None => MaxTrackSizingFunction::fit_content_px(limit.pixels),
            Some(fraction) if limit.pixels == 0.0 => MaxTrackSizingFunction::fit_content_percent(fraction),
            Some(_) => MaxTrackSizingFunction::fit_content_calc(place.to_pointer()),
        }
    }
}
