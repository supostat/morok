use crate::input::key_chord::KeyChord;
use crate::input::shortcut_id::ShortcutId;
use crate::input::shortcut_scope::ShortcutScope;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Shortcut {
    pub(crate) id: ShortcutId,
    pub(crate) scope: ShortcutScope,
    pub(crate) chord: KeyChord,
}
