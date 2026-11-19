use crate::input::key_chord::KeyChord;
use crate::input::shortcut_id::ShortcutId;
use crate::input::shortcut_scope::ShortcutScope;
use crate::menu::menu_command::MenuCommand;

#[derive(Clone, Debug)]
pub(crate) struct Shortcut {
    pub(crate) id: ShortcutId,
    pub(crate) scope: ShortcutScope,
    pub(crate) chord: KeyChord,
    /// The line of the menu bar that shows the shortcut; none for one the menu bar leaves out.
    pub(crate) menu_command: Option<MenuCommand>,
}
