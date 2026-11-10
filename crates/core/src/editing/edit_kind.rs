/// What kind of change an edit is to the history of a text: successive edits of one kind
/// that go on where the last ended are undone together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EditKind {
    /// Text typed at the caret.
    Typing,
    /// The character before the caret deleted.
    DeletingBackward,
    /// The character after the caret deleted.
    DeletingForward,
    /// A change that is undone by itself: a paste, a cut, a word deleted.
    Single,
}
