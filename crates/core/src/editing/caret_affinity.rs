/// Which of two lines a caret belongs to where a line wraps: the end of the upper line and the
/// start of the lower one are the same place in the text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum CaretAffinity {
    /// The caret stands at the end of the upper line.
    Upstream,
    /// The caret stands at the start of the lower line.
    #[default]
    Downstream,
}
