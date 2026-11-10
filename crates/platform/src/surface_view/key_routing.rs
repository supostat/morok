/// What became of a key the view handed to the input methods of the system.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum KeyRouting {
    /// Neither text nor a command came of it yet.
    #[default]
    Waiting,
    /// It typed text, or the content used it as a command.
    Used,
    /// It was a command the content did not use.
    Unused,
}
