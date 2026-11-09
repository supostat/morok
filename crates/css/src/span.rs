use cssparser::SourcePosition;

/// A byte range of a source file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Span {
    pub(crate) const FILE_START: Self = Self { start: 0, end: 0 };

    pub(crate) fn between(start: SourcePosition, end: SourcePosition) -> Self {
        Self {
            start: start.byte_index(),
            end: end.byte_index(),
        }
    }

    pub(crate) fn is_empty(self) -> bool {
        self.start == self.end
    }

    pub(crate) fn without_surrounding_whitespace(self, text: &str) -> Self {
        let covered = &text[self.start..self.end];
        let start = self.start + (covered.len() - covered.trim_start().len());
        Self {
            start,
            end: start + covered.trim().len(),
        }
    }
}
