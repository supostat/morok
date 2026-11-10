use crate::editing::movement::Movement;
use crate::editing::text_command::TextCommand;

/// What the keyboard asks of an edited text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EditCommand {
    /// Moves the caret; a selection gives way to a caret at the end the movement points to.
    Move(Movement),
    /// Moves the head of the selection and leaves its anchor.
    Extend(Movement),
    /// Deletes the selection, or with a caret what the movement would pass over.
    Delete(Movement),
    InsertLineBreak,
    /// Breaks the line after the caret and leaves the caret where it is.
    OpenLine,
    /// Swaps the characters on either side of the caret.
    Transpose,
    SelectAll,
    Copy,
    Cut,
    Paste,
    Undo,
    Redo,
    /// Shows the start of the text and leaves the caret where it is.
    ScrollToStart,
    /// Shows the end of the text and leaves the caret where it is.
    ScrollToEnd,
    ScrollPageUp,
    ScrollPageDown,
}

impl From<TextCommand> for EditCommand {
    fn from(command: TextCommand) -> Self {
        match command {
            TextCommand::Copy => Self::Copy,
            TextCommand::Cut => Self::Cut,
            TextCommand::Paste => Self::Paste,
            TextCommand::SelectAll => Self::SelectAll,
            TextCommand::Undo => Self::Undo,
            TextCommand::Redo => Self::Redo,
        }
    }
}
