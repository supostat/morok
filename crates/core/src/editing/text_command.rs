/// What the commands of the Edit menu ask of the focused text, whichever way they are given.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextCommand {
    Copy,
    Cut,
    Paste,
    SelectAll,
    Undo,
    Redo,
}
