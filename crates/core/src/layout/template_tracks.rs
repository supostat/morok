use std::iter::Enumerate;
use std::slice;

use morok_style::TrackListItem;
use taffy::GenericGridTemplateComponent;

use crate::layout::track_repetition::TrackRepetition;
use crate::layout::track_sizing::TrackSizing;

/// The entries of `grid-template-columns` or `grid-template-rows` as Taffy reads them.
#[derive(Clone, Debug)]
pub(crate) struct TemplateTracks<'a> {
    entries: Enumerate<slice::Iter<'a, TrackListItem>>,
    sizing: TrackSizing<'a>,
}

impl<'a> TemplateTracks<'a> {
    pub(crate) fn new(entries: &'a [TrackListItem], sizing: TrackSizing<'a>) -> Self {
        Self {
            entries: entries.iter().enumerate(),
            sizing,
        }
    }
}

impl<'a> Iterator for TemplateTracks<'a> {
    type Item = GenericGridTemplateComponent<String, TrackRepetition<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let (entry, item) = self.entries.next()?;
        Some(match item {
            TrackListItem::Track(track) => {
                GenericGridTemplateComponent::Single(self.sizing.sizing_function(track, entry, 0))
            }
            TrackListItem::Repeat { count, tracks } => GenericGridTemplateComponent::Repeat(TrackRepetition {
                count: *count,
                tracks,
                sizing: self.sizing,
                entry,
            }),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.entries.size_hint()
    }
}

impl ExactSizeIterator for TemplateTracks<'_> {}
