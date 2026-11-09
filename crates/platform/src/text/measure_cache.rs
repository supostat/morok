use std::collections::HashMap;

use morok_core::{FontId, TextExtent, WidthConstraint};

/// Past this many answers the cache starts over: texts that are no longer shown must not
/// stay for good.
const MAXIMUM_ANSWERS: usize = 4096;

/// How a text is set, apart from the text itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MeasureKey {
    font: FontId,
    letter_spacing: u32,
    /// The kind of the width constraint and, for a definite one, its width.
    width: (u8, u32),
}

impl MeasureKey {
    pub(crate) fn new(font: FontId, letter_spacing: f32, width: WidthConstraint) -> Self {
        Self {
            font,
            letter_spacing: letter_spacing.to_bits(),
            width: match width {
                WidthConstraint::MinContent => (0, 0),
                WidthConstraint::MaxContent => (1, 0),
                WidthConstraint::Definite(width) => (2, width.to_bits()),
            },
        }
    }
}

/// The measurements a shaper already made, by string, style and width. Layout asks about the
/// same text several times in one frame and again in every frame that lays it out.
#[derive(Debug, Default)]
pub(crate) struct MeasureCache {
    answers: HashMap<MeasureKey, HashMap<Box<str>, TextExtent>>,
    answer_count: usize,
}

impl MeasureCache {
    pub(crate) fn get(&self, key: MeasureKey, text: &str) -> Option<TextExtent> {
        self.answers.get(&key)?.get(text).copied()
    }

    pub(crate) fn insert(&mut self, key: MeasureKey, text: &str, extent: TextExtent) {
        if self.answer_count == MAXIMUM_ANSWERS {
            self.answers.clear();
            self.answer_count = 0;
        }
        if self
            .answers
            .entry(key)
            .or_default()
            .insert(text.into(), extent)
            .is_none()
        {
            self.answer_count += 1;
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.answer_count
    }
}
