/// A comment with its delimiters and the line it starts on.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Comment {
    pub(crate) line: usize,
    pub(crate) text: String,
}
