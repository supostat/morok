#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    /// A key that types a character, named by the lower-case character it types with no
    /// modifier held. The space bar is [`Key::Space`].
    Character(char),
    Enter,
    Tab,
    Space,
    Escape,
    Backspace,
    Delete,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    PageUp,
    PageDown,
    /// A function key by its number: `Function(5)` is F5.
    Function(u8),
}
