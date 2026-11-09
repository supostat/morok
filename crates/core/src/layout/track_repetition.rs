use std::iter::Map;
use std::slice;

use morok_style::{RepeatCount, TrackSize};
use taffy::{GenericRepetition, RepetitionCount};

use crate::layout::track_sizes::TrackSizes;
use crate::layout::track_sizing::TrackSizing;

/// The line names of a track list, in the shape Taffy asks for. The subset has no named lines,
/// so the list is always empty.
pub(crate) type LineNames<'a> = Map<slice::Iter<'a, Vec<String>>, fn(&Vec<String>) -> slice::Iter<'_, String>>;

pub(crate) fn no_line_names<'a>() -> LineNames<'a> {
    let no_names: &[Vec<String>] = &[];
    no_names.iter().map(|names| names.iter())
}

/// A `repeat()` entry of a track list.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TrackRepetition<'a> {
    pub(crate) count: RepeatCount,
    pub(crate) tracks: &'a [TrackSize],
    pub(crate) sizing: TrackSizing<'a>,
    /// The position of the entry in its list.
    pub(crate) entry: usize,
}

impl GenericRepetition for TrackRepetition<'_> {
    type CustomIdent = String;

    type RepetitionTrackList<'a>
        = TrackSizes<'a>
    where
        Self: 'a;

    type TemplateLineNames<'a>
        = LineNames<'a>
    where
        Self: 'a;

    fn count(&self) -> RepetitionCount {
        match self.count {
            RepeatCount::Times(times) => RepetitionCount::Count(times),
            RepeatCount::AutoFill => RepetitionCount::AutoFill,
            RepeatCount::AutoFit => RepetitionCount::AutoFit,
        }
    }

    fn tracks(&self) -> Self::RepetitionTrackList<'_> {
        TrackSizes::repeated(self.tracks, self.sizing, self.entry)
    }

    fn lines_names(&self) -> Self::TemplateLineNames<'_> {
        no_line_names()
    }
}
