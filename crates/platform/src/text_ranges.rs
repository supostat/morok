use std::ops::Range;

use objc2_foundation::{NSNotFound, NSRange};

/// The range of no characters at all, as `AppKit` gives and takes it.
pub(crate) fn no_range() -> NSRange {
    NSRange::new(not_found(), 0)
}

fn not_found() -> usize {
    usize::try_from(NSNotFound).expect("NSNotFound is not negative")
}

/// The bytes of a text that a range of its UTF-16 code units covers, each end moved back to
/// the start of the character it falls into and kept inside the text; none for a range at
/// `NSNotFound`.
pub(crate) fn byte_range(text: &str, range: NSRange) -> Option<Range<usize>> {
    if range.location == not_found() {
        return None;
    }
    let start = byte_of(text, range.location);
    let end = byte_of(text, range.location.saturating_add(range.length));
    Some(start..end.max(start))
}

/// The UTF-16 code units a range of the bytes of a text covers.
pub(crate) fn utf16_range(text: &str, range: &Range<usize>) -> NSRange {
    let start = utf16_offset(text, range.start);
    NSRange::new(start, utf16_offset(text, range.end) - start)
}

/// The byte the code unit at `utf16` starts, or the end of the text past its last.
pub(crate) fn byte_of(text: &str, utf16: usize) -> usize {
    let mut counted = 0;
    for (position, character) in text.char_indices() {
        if counted + character.len_utf16() > utf16 {
            return position;
        }
        counted += character.len_utf16();
    }
    text.len()
}

pub(crate) fn utf16_offset(text: &str, byte: usize) -> usize {
    text[..byte].chars().map(char::len_utf16).sum()
}

#[cfg(test)]
mod tests {
    use objc2_foundation::NSRange;

    use super::{byte_range, no_range, utf16_range};

    #[test]
    fn ranges_of_code_units_and_of_bytes_name_the_same_characters() {
        let text = "a\u{1F600}b\u{E9}";

        assert_eq!(utf16_range(text, &(1..6)), NSRange::new(1, 3));
        assert_eq!(byte_range(text, NSRange::new(1, 3)), Some(1..6));
        assert_eq!(
            byte_range(text, NSRange::new(2, 1)),
            Some(1..5),
            "half a surrogate pair falls back on its start"
        );
        assert_eq!(byte_range(text, NSRange::new(4, 40)), Some(6..8));
        assert_eq!(byte_range(text, no_range()), None);
    }
}
