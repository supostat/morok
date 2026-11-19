use morok_core::{Key, KeyChord, Modifiers};

/// The modifiers in the order the system writes them, each with its sign.
const MODIFIER_SIGNS: [(Modifiers, char); 4] = [
    (Modifiers::CONTROL, '⌃'),
    (Modifiers::OPTION, '⌥'),
    (Modifiers::SHIFT, '⇧'),
    (Modifiers::COMMAND, '⌘'),
];

/// How a chord reads beside the item of a menu: the signs of its modifiers, then the sign or
/// the capital of its key.
pub(crate) fn shortcut_label(chord: KeyChord) -> String {
    let mut label: String = MODIFIER_SIGNS
        .iter()
        .filter(|(modifier, _)| chord.modifiers.contains(*modifier))
        .map(|(_, sign)| *sign)
        .collect();
    match chord.key {
        Key::Character(character) => label.extend(character.to_uppercase()),
        Key::Function(number) => {
            label.push('F');
            label.push_str(&number.to_string());
        }
        Key::Enter => label.push('↩'),
        Key::Tab => label.push('⇥'),
        Key::Space => label.push('␣'),
        Key::Escape => label.push('⎋'),
        Key::Backspace => label.push('⌫'),
        Key::Delete => label.push('⌦'),
        Key::ArrowLeft => label.push('←'),
        Key::ArrowRight => label.push('→'),
        Key::ArrowUp => label.push('↑'),
        Key::ArrowDown => label.push('↓'),
        Key::Home => label.push('↖'),
        Key::End => label.push('↘'),
        Key::PageUp => label.push('⇞'),
        Key::PageDown => label.push('⇟'),
    }
    label
}

#[cfg(test)]
mod tests {
    use morok_core::{Key, KeyChord, Modifiers};

    use super::shortcut_label;

    #[test]
    fn a_chord_reads_as_its_modifiers_in_the_order_of_the_system_and_its_key() {
        let copy_path = KeyChord {
            key: Key::Character('c'),
            modifiers: Modifiers::COMMAND.union(Modifiers::OPTION),
        };
        let move_to_trash = KeyChord {
            key: Key::Backspace,
            modifiers: Modifiers::COMMAND,
        };
        let quick_look = KeyChord {
            key: Key::Space,
            modifiers: Modifiers::NONE,
        };
        let refresh = KeyChord {
            key: Key::Function(5),
            modifiers: Modifiers::SHIFT.union(Modifiers::CONTROL),
        };

        assert_eq!(shortcut_label(copy_path), "⌥⌘C");
        assert_eq!(shortcut_label(move_to_trash), "⌘⌫");
        assert_eq!(shortcut_label(quick_look), "␣");
        assert_eq!(shortcut_label(refresh), "⌃⇧F5");
    }
}
