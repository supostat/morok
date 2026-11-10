use crate::editing::edit_command::EditCommand;
use crate::editing::line_mode::LineMode;
use crate::editing::movement::Movement;
use crate::input::key::Key;
use crate::input::key_event::KeyEvent;
use crate::input::modifiers::Modifiers;

const SHIFT_OPTION: Modifiers = Modifiers::SHIFT.union(Modifiers::OPTION);
const SHIFT_COMMAND: Modifiers = Modifiers::SHIFT.union(Modifiers::COMMAND);
const SHIFT_CONTROL: Modifiers = Modifiers::SHIFT.union(Modifiers::CONTROL);

/// What a key asks of an edited text, by the standard key bindings of macOS; none for a key
/// that edits nothing, which goes on to the elements around the text. A key that types a
/// character is not among them: typed text comes from the input methods of the system.
pub(crate) fn edit_command(event: &KeyEvent, line_mode: LineMode) -> Option<EditCommand> {
    if !event.is_press() {
        return None;
    }
    let modifiers = event.modifiers;
    match event.key {
        Key::ArrowLeft => horizontal(
            modifiers,
            Movement::GraphemeBackward,
            Movement::WordBackward,
            Movement::LineStart,
        ),
        Key::ArrowRight => horizontal(
            modifiers,
            Movement::GraphemeForward,
            Movement::WordForward,
            Movement::LineEnd,
        ),
        Key::ArrowUp => vertical(
            modifiers,
            Movement::LineUp,
            Movement::ParagraphBackward,
            Movement::DocumentStart,
        ),
        Key::ArrowDown => vertical(
            modifiers,
            Movement::LineDown,
            Movement::ParagraphForward,
            Movement::DocumentEnd,
        ),
        Key::Home => scrolling(modifiers, EditCommand::ScrollToStart, Movement::DocumentStart, None),
        Key::End => scrolling(modifiers, EditCommand::ScrollToEnd, Movement::DocumentEnd, None),
        Key::PageUp => scrolling(
            modifiers,
            EditCommand::ScrollPageUp,
            Movement::PageUp,
            Some(Movement::PageUp),
        ),
        Key::PageDown => scrolling(
            modifiers,
            EditCommand::ScrollPageDown,
            Movement::PageDown,
            Some(Movement::PageDown),
        ),
        Key::Backspace => deletion(
            modifiers,
            Movement::GraphemeBackward,
            Movement::WordBackward,
            Movement::LineStart,
        ),
        Key::Delete => deletion(
            modifiers,
            Movement::GraphemeForward,
            Movement::WordForward,
            Movement::LineEnd,
        ),
        Key::Enter => line_break(modifiers, line_mode),
        Key::Character(character) => character_command(character, modifiers, line_mode),
        Key::Tab | Key::Space | Key::Escape | Key::Function(_) => None,
    }
}

/// The left and right arrows: by character, with Option by word, with Command or Control to
/// the end of the line; Shift extends the selection.
fn horizontal(
    modifiers: Modifiers,
    by_character: Movement,
    by_word: Movement,
    to_line_end: Movement,
) -> Option<EditCommand> {
    let (extends, others) = without_shift(modifiers);
    let movement = if others.is_empty() {
        by_character
    } else if others == Modifiers::OPTION {
        by_word
    } else if others == Modifiers::COMMAND || others == Modifiers::CONTROL {
        to_line_end
    } else {
        return None;
    };
    Some(moved(movement, extends))
}

/// The up and down arrows: by line, with Option by paragraph, with Command to the end of the
/// text; Shift extends the selection.
fn vertical(
    modifiers: Modifiers,
    by_line: Movement,
    by_paragraph: Movement,
    to_document_end: Movement,
) -> Option<EditCommand> {
    let (extends, others) = without_shift(modifiers);
    let movement = if others.is_empty() {
        by_line
    } else if others == Modifiers::OPTION {
        by_paragraph
    } else if others == Modifiers::COMMAND {
        to_document_end
    } else {
        return None;
    };
    Some(moved(movement, extends))
}

/// Home, End and the page keys scroll and leave the caret; with Shift they extend the
/// selection, and the page keys with Option move the caret.
fn scrolling(
    modifiers: Modifiers,
    scroll: EditCommand,
    extended_by: Movement,
    moved_with_option: Option<Movement>,
) -> Option<EditCommand> {
    if modifiers.is_empty() {
        Some(scroll)
    } else if modifiers == Modifiers::SHIFT {
        Some(EditCommand::Extend(extended_by))
    } else if modifiers == Modifiers::OPTION {
        moved_with_option.map(EditCommand::Move)
    } else if modifiers == SHIFT_OPTION {
        moved_with_option.map(EditCommand::Extend)
    } else {
        None
    }
}

/// The delete keys: a character, with Option a word, with Command to the end of the line.
/// Shift and Control change nothing of a plain deletion.
fn deletion(
    modifiers: Modifiers,
    by_character: Movement,
    by_word: Movement,
    to_line_end: Movement,
) -> Option<EditCommand> {
    let (_, others) = without_shift(modifiers);
    let movement = if others.is_empty() || others == Modifiers::CONTROL {
        by_character
    } else if others == Modifiers::OPTION {
        by_word
    } else if others == Modifiers::COMMAND {
        to_line_end
    } else {
        return None;
    };
    Some(EditCommand::Delete(movement))
}

/// Return breaks a line where the text has lines; in one line it is left to the element
/// around the text, which may take it as the end of the input.
fn line_break(modifiers: Modifiers, line_mode: LineMode) -> Option<EditCommand> {
    let breaks = modifiers.is_empty() || modifiers == Modifiers::SHIFT || modifiers == Modifiers::OPTION;
    (line_mode == LineMode::Multiple && breaks).then_some(EditCommand::InsertLineBreak)
}

fn character_command(character: char, modifiers: Modifiers, line_mode: LineMode) -> Option<EditCommand> {
    if modifiers == Modifiers::COMMAND {
        return match character {
            'a' => Some(EditCommand::SelectAll),
            'c' => Some(EditCommand::Copy),
            'x' => Some(EditCommand::Cut),
            'v' => Some(EditCommand::Paste),
            'z' => Some(EditCommand::Undo),
            _ => None,
        };
    }
    if modifiers == SHIFT_COMMAND {
        return (character == 'z').then_some(EditCommand::Redo);
    }
    let extends = modifiers == SHIFT_CONTROL;
    if modifiers != Modifiers::CONTROL && !extends {
        return None;
    }
    let movement = match character {
        'a' => Movement::ParagraphStart,
        'e' => Movement::ParagraphEnd,
        'f' => Movement::GraphemeForward,
        'b' => Movement::GraphemeBackward,
        'n' => Movement::LineDown,
        'p' => Movement::LineUp,
        _ if extends => return None,
        'd' => return Some(EditCommand::Delete(Movement::GraphemeForward)),
        'h' => return Some(EditCommand::Delete(Movement::GraphemeBackward)),
        'k' => return Some(EditCommand::Delete(Movement::ParagraphEnd)),
        't' => return Some(EditCommand::Transpose),
        'o' => return (line_mode == LineMode::Multiple).then_some(EditCommand::OpenLine),
        _ => return None,
    };
    Some(moved(movement, extends))
}

fn moved(movement: Movement, extends: bool) -> EditCommand {
    if extends {
        EditCommand::Extend(movement)
    } else {
        EditCommand::Move(movement)
    }
}

/// Whether Shift is held, and the modifiers held besides it.
fn without_shift(modifiers: Modifiers) -> (bool, Modifiers) {
    let extends = modifiers.contains(Modifiers::SHIFT);
    let others = [Modifiers::CONTROL, Modifiers::OPTION, Modifiers::COMMAND]
        .into_iter()
        .filter(|modifier| modifiers.contains(*modifier))
        .fold(Modifiers::NONE, Modifiers::union);
    (extends, others)
}
