use crate::span::Span;

/// A rejected construct with the place and the words of its diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Problem {
    pub(crate) span: Span,
    pub(crate) message: String,
    pub(crate) hint: String,
}
