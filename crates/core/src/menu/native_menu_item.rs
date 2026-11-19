use crate::input::key_chord::KeyChord;

/// One line of a [`NativeMenu`](crate::NativeMenu).
#[derive(Clone, Debug, PartialEq)]
pub enum NativeMenuItem {
    /// A line the user chooses.
    Command {
        /// Names the line to whoever asked for the menu: the host tells it which was chosen.
        id: usize,
        label: String,
        is_disabled: bool,
        /// The chord shown beside the label.
        shortcut: Option<KeyChord>,
    },
    /// A line that opens a menu of its own.
    Submenu {
        label: String,
        items: Vec<Self>,
    },
    Separator,
}
