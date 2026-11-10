use std::ops::Range;

use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};

/// The nearest place at or before `offset` a caret may stand at: the edge of a grapheme
/// cluster, so that no caret splits a character with its combining marks or an emoji sequence.
pub(crate) fn floor_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    let is_boundary = GraphemeCursor::new(offset, text.len(), true)
        .is_boundary(text, 0)
        .unwrap_or(true);
    if is_boundary {
        offset
    } else {
        previous_grapheme(text, offset)
    }
}

/// The edge of a grapheme cluster after `offset`; the end of the text when there is none.
pub(crate) fn next_grapheme(text: &str, offset: usize) -> usize {
    GraphemeCursor::new(offset, text.len(), true)
        .next_boundary(text, 0)
        .ok()
        .flatten()
        .unwrap_or(text.len())
}

/// The edge of a grapheme cluster before `offset`; the start of the text when there is none.
pub(crate) fn previous_grapheme(text: &str, offset: usize) -> usize {
    GraphemeCursor::new(offset, text.len(), true)
        .prev_boundary(text, 0)
        .ok()
        .flatten()
        .unwrap_or(0)
}

/// The end of the word `offset` is in, or of the next word when it is between words; the end
/// of the text after the last word. White space and punctuation are no words.
pub(crate) fn word_end_after(text: &str, offset: usize) -> usize {
    text.unicode_word_indices()
        .map(|(start, word)| start + word.len())
        .find(|end| *end > offset)
        .unwrap_or(text.len())
}

/// The start of the word `offset` is in, or of the word before when it is between words; the
/// start of the text before the first word.
pub(crate) fn word_start_before(text: &str, offset: usize) -> usize {
    text.unicode_word_indices()
        .map(|(start, _)| start)
        .take_while(|start| *start < offset)
        .last()
        .unwrap_or(0)
}

/// The stretch of the text a double click at `offset` selects: the word there, or the run of
/// white space or the punctuation mark between two words.
pub(crate) fn word_around(text: &str, offset: usize) -> Range<usize> {
    text.split_word_bound_indices()
        .map(|(start, segment)| start..start + segment.len())
        .take_while(|segment| segment.start <= offset)
        .last()
        .unwrap_or(0..0)
}

/// The start of the paragraph `offset` is in: after the line break before it.
pub(crate) fn paragraph_start(text: &str, offset: usize) -> usize {
    text[..offset].rfind('\n').map_or(0, |line_break| line_break + 1)
}

/// The end of the paragraph `offset` is in: before the line break that ends it.
pub(crate) fn paragraph_end(text: &str, offset: usize) -> usize {
    text[offset..]
        .find('\n')
        .map_or(text.len(), |line_break| offset + line_break)
}

/// The place `count` grapheme clusters after `start`, no further than `end`.
pub(crate) fn graphemes_after(text: &str, start: usize, end: usize, count: usize) -> usize {
    text[start..end]
        .grapheme_indices(true)
        .nth(count)
        .map_or(end, |(position, _)| start + position)
}

#[cfg(test)]
mod tests {
    use super::{
        floor_boundary, next_grapheme, paragraph_end, paragraph_start, previous_grapheme, word_around, word_end_after,
        word_start_before,
    };

    const FAMILY: &str = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";

    #[test]
    fn a_grapheme_cluster_is_stepped_over_as_one_character() {
        let text = format!("e\u{301}{FAMILY}x");
        let accented = "e\u{301}".len();
        let after_family = accented + FAMILY.len();

        assert_eq!(next_grapheme(&text, 0), accented);
        assert_eq!(next_grapheme(&text, accented), after_family);
        assert_eq!(previous_grapheme(&text, after_family), accented);
        assert_eq!(previous_grapheme(&text, accented), 0);
        assert_eq!(next_grapheme(&text, text.len()), text.len());
        assert_eq!(previous_grapheme(&text, 0), 0);
    }

    #[test]
    fn an_offset_inside_a_cluster_falls_back_on_its_start() {
        let text = format!("a{FAMILY}");

        assert_eq!(floor_boundary(&text, 2), 1);
        assert_eq!(floor_boundary(&text, 1 + "\u{1F468}".len()), 1);
        assert_eq!(floor_boundary(&text, 400), text.len());
    }

    #[test]
    fn word_movement_skips_white_space_and_punctuation() {
        let text = "one, two  three";

        assert_eq!(word_end_after(text, 0), 3);
        assert_eq!(word_end_after(text, 3), 8);
        assert_eq!(word_end_after(text, 9), 15);
        assert_eq!(word_end_after(text, 15), 15);
        assert_eq!(word_start_before(text, 15), 10);
        assert_eq!(word_start_before(text, 10), 5);
        assert_eq!(word_start_before(text, 4), 0);
        assert_eq!(word_start_before(text, 0), 0);
    }

    #[test]
    fn a_double_click_takes_the_word_or_what_lies_between_two_words() {
        let text = "one, two";

        assert_eq!(word_around(text, 1), 0..3);
        assert_eq!(word_around(text, 3), 3..4);
        assert_eq!(word_around(text, 8), 5..8);
        assert_eq!(word_around("", 0), 0..0);
    }

    #[test]
    fn a_paragraph_lies_between_two_line_breaks() {
        let text = "one\ntwo\n";

        assert_eq!((paragraph_start(text, 5), paragraph_end(text, 5)), (4, 7));
        assert_eq!((paragraph_start(text, 3), paragraph_end(text, 3)), (0, 3));
        assert_eq!((paragraph_start(text, 8), paragraph_end(text, 8)), (8, 8));
    }
}
