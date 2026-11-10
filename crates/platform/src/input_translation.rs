use morok_core::{Key, KeyEvent, KeyPhase, Modifiers, Point, PointerInput, PointerInputKind};
use objc2_app_kit::{NSEvent, NSEventModifierFlags, NSView};

/// The keys that are told by where they sit on the keyboard, by their virtual key code.
const NAMED_KEYS: [(u16, Key); 28] = [
    (0x24, Key::Enter),
    (0x4C, Key::Enter),
    (0x30, Key::Tab),
    (0x31, Key::Space),
    (0x35, Key::Escape),
    (0x33, Key::Backspace),
    (0x75, Key::Delete),
    (0x7B, Key::ArrowLeft),
    (0x7C, Key::ArrowRight),
    (0x7E, Key::ArrowUp),
    (0x7D, Key::ArrowDown),
    (0x73, Key::Home),
    (0x77, Key::End),
    (0x74, Key::PageUp),
    (0x79, Key::PageDown),
    (0x7A, Key::Function(1)),
    (0x78, Key::Function(2)),
    (0x63, Key::Function(3)),
    (0x76, Key::Function(4)),
    (0x60, Key::Function(5)),
    (0x61, Key::Function(6)),
    (0x62, Key::Function(7)),
    (0x64, Key::Function(8)),
    (0x65, Key::Function(9)),
    (0x6D, Key::Function(10)),
    (0x67, Key::Function(11)),
    (0x6F, Key::Function(12)),
    (0x69, Key::Function(13)),
];

/// What the pointer did on the surface of `view`, from the mouse event `AppKit` sent for it.
pub(crate) fn pointer_input(view: &NSView, event: &NSEvent, kind: PointerInputKind) -> PointerInput {
    // A flipped view measures from its top left corner in points, as its surface does in CSS
    // pixels.
    let in_view = view.convertPoint_fromView(event.locationInWindow(), None);
    PointerInput {
        kind,
        position: Point {
            x: in_view.x as f32,
            y: in_view.y as f32,
        },
        modifiers: modifiers(event.modifierFlags()),
    }
}

/// How many presses in quick succession a mouse-down or mouse-up event completes.
pub(crate) fn click_count(event: &NSEvent) -> u32 {
    u32::try_from(event.clickCount()).expect("the click count of a mouse event is not negative")
}

/// The key of a key-down or key-up event; none for a key the kit has no name for.
pub(crate) fn key_event(event: &NSEvent, phase: KeyPhase) -> Option<KeyEvent> {
    Some(KeyEvent {
        phase,
        key: key(event)?,
        modifiers: modifiers(event.modifierFlags()),
        is_repeat: event.isARepeat(),
    })
}

fn modifiers(flags: NSEventModifierFlags) -> Modifiers {
    [
        (NSEventModifierFlags::Shift, Modifiers::SHIFT),
        (NSEventModifierFlags::Control, Modifiers::CONTROL),
        (NSEventModifierFlags::Option, Modifiers::OPTION),
        (NSEventModifierFlags::Command, Modifiers::COMMAND),
    ]
    .into_iter()
    .filter(|(flag, _)| flags.contains(*flag))
    .fold(Modifiers::NONE, |held, (_, modifier)| held.union(modifier))
}

fn key(event: &NSEvent) -> Option<Key> {
    let key_code = event.keyCode();
    NAMED_KEYS
        .iter()
        .find(|(code, _)| *code == key_code)
        .map(|(_, key)| *key)
        .or_else(|| typed_character(event).map(Key::Character))
}

/// The character the key types with no modifier held, in lower case; none for a key that
/// types none, which `AppKit` gives a code of the private use area or a control code.
fn typed_character(event: &NSEvent) -> Option<char> {
    let characters = event.charactersIgnoringModifiers()?;
    if characters.length() == 0 {
        return None;
    }
    let character = char::from_u32(u32::from(characters.characterAtIndex(0)))?;
    let types_nothing = character.is_control() || ('\u{E000}'..='\u{F8FF}').contains(&character);
    if types_nothing {
        return None;
    }
    character.to_lowercase().next()
}
