use std::fmt::Debug;

/// The port through which a tree reaches the clipboard of the system. The tree knows nothing
/// of pasteboards: the platform implements the port.
pub trait Clipboard: Debug {
    /// The text the clipboard holds; none when it holds no text.
    fn text(&mut self) -> Option<String>;

    /// Puts the text on the clipboard in place of what it held.
    fn set_text(&mut self, text: &str);
}
