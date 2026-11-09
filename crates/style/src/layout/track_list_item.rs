use std::borrow::Cow;

use crate::layout::repeat_count::RepeatCount;
use crate::layout::track_size::TrackSize;

/// One entry of `grid-template-columns` or `grid-template-rows`.
#[derive(Clone, Debug, PartialEq)]
pub enum TrackListItem {
    Track(TrackSize),
    Repeat {
        count: RepeatCount,
        tracks: Cow<'static, [TrackSize]>,
    },
}
