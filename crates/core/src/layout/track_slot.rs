use crate::layout::track_bound::TrackBound;
use crate::layout::track_list::TrackList;

/// The place of one length inside a list of grid tracks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TrackSlot {
    pub(crate) list: TrackList,
    /// The position of the entry in the list.
    pub(crate) entry: usize,
    /// The position of the track inside a `repeat()` entry; zero for a single track.
    pub(crate) repeated: usize,
    pub(crate) bound: TrackBound,
}
