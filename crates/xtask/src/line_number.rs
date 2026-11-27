/// The number of the first line of a text.
pub(crate) const FIRST: usize = 1;

/// The number of the line of `text` that holds the byte at `offset`.
pub(crate) fn at(text: &str, offset: usize) -> usize {
    text[..offset].matches('\n').count() + FIRST
}
