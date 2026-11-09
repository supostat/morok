use std::iter::Enumerate;
use std::slice;

use morok_style::TrackSize;
use taffy::TrackSizingFunction;

use crate::layout::track_sizing::TrackSizing;

/// The sizing functions of consecutive tracks: a whole list of implicit tracks, or the tracks of
/// one `repeat()` entry.
#[derive(Clone, Debug)]
pub(crate) struct TrackSizes<'a> {
    tracks: Enumerate<slice::Iter<'a, TrackSize>>,
    sizing: TrackSizing<'a>,
    /// The entry of the list the tracks repeat in; without one every track is an entry of its own.
    repeat_entry: Option<usize>,
}

impl<'a> TrackSizes<'a> {
    pub(crate) fn entries(tracks: &'a [TrackSize], sizing: TrackSizing<'a>) -> Self {
        Self {
            tracks: tracks.iter().enumerate(),
            sizing,
            repeat_entry: None,
        }
    }

    pub(crate) fn repeated(tracks: &'a [TrackSize], sizing: TrackSizing<'a>, repeat_entry: usize) -> Self {
        Self {
            tracks: tracks.iter().enumerate(),
            sizing,
            repeat_entry: Some(repeat_entry),
        }
    }
}

impl Iterator for TrackSizes<'_> {
    type Item = TrackSizingFunction;

    fn next(&mut self) -> Option<TrackSizingFunction> {
        let (position, track) = self.tracks.next()?;
        Some(match self.repeat_entry {
            Some(entry) => self.sizing.sizing_function(track, entry, position),
            None => self.sizing.sizing_function(track, position, 0),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.tracks.size_hint()
    }
}

impl ExactSizeIterator for TrackSizes<'_> {}
