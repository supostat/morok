mod support;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use morok_core::accesskit::{
    Action, ActionData, ActionRequest, NodeId, Role as AccessibleRole, TextPosition, TextSelection as NodeSelection,
    TreeId, TreeUpdate,
};
use morok_core::{
    Clipboard, DisplayItem, ElementId, Focusability, Key, KeyEvent, KeyPhase, Modifiers, Point, PointerButton,
    PointerInput, PointerInputKind, Rect, Role, TextSelection,
};
use morok_style::Color;
use support::{CHARACTER_WIDTH, InputLog, LINE_HEIGHT, Scene};

/// A field ten characters wide, an area ten characters wide and two lines high, and an area
/// that is as high as its lines.
const RULES: &str = ".field { width: 100px; color: var(--color-text); caret-color: var(--color-accent); }\n\
    .field::placeholder { color: var(--color-text-3); }\n\
    .field::selection { background-color: var(--color-selection); }\n\
    .area { width: 100px; height: 40px; }\n\
    .leading { width: 100px; line-height: var(--text-leading); }\n\
    .growing { width: 100px; }";

const SHIFT: Modifiers = Modifiers::SHIFT;
const OPTION: Modifiers = Modifiers::OPTION;
const COMMAND: Modifiers = Modifiers::COMMAND;
const CONTROL: Modifiers = Modifiers::CONTROL;
const NONE: Modifiers = Modifiers::NONE;

/// A man, a woman and a girl joined into one family: one grapheme cluster of eighteen bytes.
const FAMILY: &str = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";

/// What the clipboard of a test holds, shared with the tree.
#[derive(Clone, Debug, Default)]
struct SharedClipboard(Rc<RefCell<Option<String>>>);

impl Clipboard for SharedClipboard {
    fn text(&mut self) -> Option<String> {
        self.0.borrow().clone()
    }

    fn set_text(&mut self, text: &str) {
        *self.0.borrow_mut() = Some(text.to_owned());
    }
}

/// A scene with one focused element that edits text, and what the tree told of its input.
struct Editing {
    scene: Scene,
    input: ElementId,
    log: InputLog,
    clipboard: SharedClipboard,
}

impl Editing {
    /// An element of the role and the class that holds `text`, focused, after its first
    /// frame: its caret stands at the end of the text.
    fn of(role: Role, class: &str, text: &str) -> Self {
        let mut scene = Scene::new(RULES);
        let input = scene.add(scene.root(), role, &[class]);
        scene.tree.set_focusability(input, Focusability::TabStop);
        scene.tree.set_text(input, text);
        assert!(scene.tree.focus(input), "an edited text takes focus");
        let clipboard = SharedClipboard::default();
        scene.tree.set_clipboard(Box::new(clipboard.clone()));
        scene.frame();
        Self {
            scene,
            input,
            log: InputLog::default(),
            clipboard,
        }
    }

    fn field(text: &str) -> Self {
        Self::of(Role::TextInput, "field", text)
    }

    fn area(text: &str) -> Self {
        Self::of(Role::TextArea, "area", text)
    }

    /// Presses a key and runs the frame that follows it. Returns whether the tree used the key.
    fn key(&mut self, key: Key, modifiers: Modifiers) -> bool {
        let was_used = self.scene.press_key(key, modifiers, &mut self.log);
        self.scene.frame();
        was_used
    }

    fn character_key(&mut self, character: char, modifiers: Modifiers) -> bool {
        self.key(Key::Character(character), modifiers)
    }

    /// Types the text a character at a time, as the keyboard gives it, with a frame after each.
    fn type_text(&mut self, text: &str) {
        for character in text.chars() {
            self.scene
                .tree
                .insert_text(self.input, character.encode_utf8(&mut [0; 4]), None, &mut self.log);
            self.scene.frame();
        }
    }

    fn select(&mut self, anchor: usize, head: usize) {
        self.scene
            .tree
            .set_text_selection(self.input, TextSelection { anchor, head });
        self.scene.frame();
    }

    fn put_caret(&mut self, offset: usize) {
        self.select(offset, offset);
    }

    fn text(&self) -> &str {
        self.scene.tree.text(self.input)
    }

    fn selection(&self) -> (usize, usize) {
        let selection = self.scene.tree.text_selection(self.input);
        (selection.anchor, selection.head)
    }

    /// Where the caret stands.
    ///
    /// # Panics
    ///
    /// When text is selected.
    fn caret(&self) -> usize {
        let selection = self.scene.tree.text_selection(self.input);
        assert!(selection.is_caret(), "{selection:?} is no caret");
        selection.head
    }

    fn scroll(&self) -> (f32, f32) {
        let scroll = self.scene.tree.text_scroll(self.input);
        (scroll.x, scroll.y)
    }

    /// The rectangles of one color the display list holds: the highlight of the selection
    /// line by line, or the caret.
    fn fills(&self) -> Vec<(Rect, Color)> {
        self.scene
            .tree
            .display_list()
            .items()
            .iter()
            .filter_map(|item| match item {
                DisplayItem::Fill { rect, color } => Some((*rect, *color)),
                _ => None,
            })
            .collect()
    }

    /// The box of the caret as the display list shows it.
    ///
    /// # Panics
    ///
    /// When the display list shows no caret, or more than a caret.
    fn shown_caret(&self) -> Rect {
        match self.fills().as_slice() {
            [(rect, _)] => *rect,
            fills => panic!("the display list shows one caret, not {fills:?}"),
        }
    }

    /// The lines the display list draws, each with where it starts on its baseline and its color.
    fn drawn_lines(&self) -> Vec<(Point, String, Color)> {
        let display_list = self.scene.tree.display_list();
        display_list
            .items()
            .iter()
            .filter_map(|item| match item {
                DisplayItem::GlyphRun {
                    origin, glyphs, color, ..
                } => Some((
                    *origin,
                    display_list
                        .glyphs(*glyphs)
                        .iter()
                        .map(|glyph| {
                            char::from_u32(u32::from(glyph.glyph)).expect("a glyph of the test font is a character")
                        })
                        .collect(),
                    *color,
                )),
                _ => None,
            })
            .collect()
    }

    fn pointer(&mut self, kind: PointerInputKind, x: f32, y: f32, modifiers: Modifiers) {
        let input = PointerInput {
            kind,
            position: Point { x, y },
            modifiers,
        };
        self.scene.tree.pointer_input(&input, &mut self.log);
        self.scene.frame();
    }

    fn press(&mut self, x: f32, y: f32, click_count: u32, modifiers: Modifiers) {
        let pressed = PointerInputKind::Pressed {
            button: PointerButton::Primary,
            click_count,
        };
        self.pointer(pressed, x, y, modifiers);
    }

    fn release(&mut self, x: f32, y: f32) {
        let released = PointerInputKind::Released {
            button: PointerButton::Primary,
            click_count: 1,
        };
        self.pointer(released, x, y, NONE);
    }

    fn accessibility_update(&mut self) -> TreeUpdate {
        self.scene.tree.accessibility_update()
    }
}

fn caret_at(column: usize, line: usize) -> Rect {
    Rect::new(
        column as f32 * CHARACTER_WIDTH,
        line as f32 * LINE_HEIGHT,
        1.0,
        LINE_HEIGHT,
    )
}

#[test]
fn the_arrows_step_over_a_grapheme_cluster_as_over_one_character() {
    let mut editing = Editing::field(&format!("e\u{301}{FAMILY}x"));
    let after_accent = "e\u{301}".len();
    let after_family = after_accent + FAMILY.len();
    editing.put_caret(0);

    assert!(editing.key(Key::ArrowRight, NONE));
    assert_eq!(editing.caret(), after_accent);
    editing.key(Key::ArrowRight, NONE);
    assert_eq!(editing.caret(), after_family);
    editing.key(Key::ArrowRight, NONE);
    editing.key(Key::ArrowRight, NONE);
    assert_eq!(editing.caret(), after_family + 1, "the end of the text stops the caret");

    editing.key(Key::ArrowLeft, NONE);
    editing.key(Key::ArrowLeft, NONE);
    assert_eq!(editing.caret(), after_accent);
    editing.character_key('b', CONTROL);
    assert_eq!(editing.caret(), 0);
    editing.character_key('f', CONTROL);
    assert_eq!(editing.caret(), after_accent);
}

#[test]
fn an_arrow_turns_a_selection_into_a_caret_at_the_end_it_points_to() {
    let mut editing = Editing::field("abcdef");

    editing.select(4, 2);
    editing.key(Key::ArrowLeft, NONE);
    assert_eq!(editing.caret(), 2);

    editing.select(2, 4);
    editing.key(Key::ArrowRight, NONE);
    assert_eq!(editing.caret(), 4);
}

#[test]
fn shift_moves_the_head_of_the_selection_and_leaves_its_anchor() {
    let mut editing = Editing::field("abcdef");
    editing.put_caret(3);

    editing.key(Key::ArrowRight, SHIFT);
    editing.key(Key::ArrowRight, SHIFT);
    assert_eq!(editing.selection(), (3, 5));

    for _ in 0..4 {
        editing.key(Key::ArrowLeft, SHIFT);
    }
    assert_eq!(
        editing.selection(),
        (3, 1),
        "the head passes the anchor to the other side"
    );

    editing.key(Key::ArrowUp, SHIFT);
    assert_eq!(editing.selection(), (3, 0));
    editing.key(Key::End, SHIFT);
    assert_eq!(editing.selection(), (3, 6));
    editing.character_key('a', COMMAND);
    assert_eq!(editing.selection(), (0, 6));
}

#[test]
fn option_moves_by_words_and_skips_what_lies_between_them() {
    let mut editing = Editing::field("one, two  three");
    editing.put_caret(0);

    let mut word_ends = Vec::new();
    for _ in 0..4 {
        editing.key(Key::ArrowRight, OPTION);
        word_ends.push(editing.caret());
    }
    assert_eq!(word_ends, [3, 8, 15, 15]);

    let mut word_starts = Vec::new();
    for _ in 0..4 {
        editing.key(Key::ArrowLeft, OPTION);
        word_starts.push(editing.caret());
    }
    assert_eq!(word_starts, [10, 5, 0, 0]);

    editing.select(1, 6);
    editing.key(Key::ArrowRight, OPTION);
    assert_eq!(
        editing.caret(),
        8,
        "a word movement starts from the end of a selection it points to"
    );

    editing.put_caret(5);
    editing.key(Key::ArrowRight, SHIFT.union(OPTION));
    assert_eq!(editing.selection(), (5, 8));
}

#[test]
fn command_moves_to_the_ends_of_the_line_and_of_the_text() {
    let mut editing = Editing::area("one\ntwo four\nsix");
    editing.put_caret(6);

    editing.key(Key::ArrowLeft, COMMAND);
    assert_eq!(editing.caret(), 4);
    editing.key(Key::ArrowRight, COMMAND);
    assert_eq!(editing.caret(), 12);
    editing.key(Key::ArrowRight, COMMAND);
    assert_eq!(editing.caret(), 12, "the end of a line is not left for the next line");

    editing.key(Key::ArrowUp, COMMAND);
    assert_eq!(editing.caret(), 0);
    editing.key(Key::ArrowDown, COMMAND);
    assert_eq!(editing.caret(), 16);
    editing.put_caret(6);
    editing.key(Key::ArrowDown, SHIFT.union(COMMAND));
    assert_eq!(editing.selection(), (6, 16));
}

#[test]
fn the_end_of_a_wrapped_line_belongs_to_that_line() {
    let mut editing = Editing::area("aaaa bbbb cccc");
    editing.put_caret(2);

    editing.key(Key::ArrowRight, COMMAND);

    assert_eq!(editing.caret(), 10);
    assert_eq!(
        editing.shown_caret(),
        Rect::new(99.0, 0.0, 1.0, LINE_HEIGHT),
        "the caret stays on the line it was on, at the edge of the box the line fills"
    );
    assert_eq!(editing.scroll(), (0.0, 0.0), "lines that wrap never scroll sideways");
    editing.key(Key::ArrowRight, COMMAND);
    assert_eq!(editing.caret(), 10, "and its line ends where it stands");
    editing.key(Key::ArrowLeft, COMMAND);
    assert_eq!(editing.caret(), 0);

    editing.put_caret(10);
    assert_eq!(
        editing.shown_caret(),
        caret_at(0, 1),
        "the same place reached another way is the start of the next line"
    );
    editing.key(Key::ArrowRight, COMMAND);
    assert_eq!(editing.caret(), 14);
}

#[test]
fn up_and_down_keep_the_column_the_caret_set_out_from() {
    let mut editing = Editing::area("aaaaaaaa\nbb\ncccccccc");
    editing.put_caret(6);

    editing.key(Key::ArrowDown, NONE);
    assert_eq!(editing.caret(), 11, "a shorter line holds the caret at its end");
    editing.key(Key::ArrowDown, NONE);
    assert_eq!(
        editing.caret(),
        18,
        "and the caret is back in its column on the line after"
    );
    editing.key(Key::ArrowUp, NONE);
    editing.key(Key::ArrowUp, NONE);
    assert_eq!(editing.caret(), 6);

    editing.key(Key::ArrowDown, NONE);
    editing.key(Key::ArrowLeft, NONE);
    editing.key(Key::ArrowDown, NONE);
    assert_eq!(editing.caret(), 13, "a sideways step sets the column anew");

    editing.put_caret(6);
    editing.character_key('n', CONTROL);
    assert_eq!(editing.caret(), 11);
    editing.character_key('p', CONTROL);
    assert_eq!(editing.caret(), 6);
}

#[test]
fn up_on_the_first_line_goes_to_the_start_of_the_text_and_down_on_the_last_to_its_end() {
    let mut editing = Editing::area("abc\ndef");
    editing.put_caret(2);
    editing.key(Key::ArrowUp, NONE);
    assert_eq!(editing.caret(), 0);
    editing.put_caret(5);
    editing.key(Key::ArrowDown, NONE);
    assert_eq!(editing.caret(), 7);

    let mut field = Editing::field("abcdef");
    field.put_caret(3);
    assert!(field.key(Key::ArrowUp, NONE));
    assert_eq!(field.caret(), 0);
    field.key(Key::ArrowDown, NONE);
    assert_eq!(field.caret(), 6);
}

#[test]
fn option_and_control_move_by_paragraphs() {
    let mut editing = Editing::area("one\ntwo\nsix");
    editing.put_caret(6);

    editing.character_key('a', CONTROL);
    assert_eq!(editing.caret(), 4);
    editing.character_key('a', CONTROL);
    assert_eq!(editing.caret(), 4, "Control-A stays at the start of the paragraph");
    editing.character_key('e', CONTROL);
    assert_eq!(editing.caret(), 7);

    editing.key(Key::ArrowUp, OPTION);
    assert_eq!(editing.caret(), 4);
    editing.key(Key::ArrowUp, OPTION);
    assert_eq!(editing.caret(), 0, "Option-Up goes on to the paragraph before");
    editing.key(Key::ArrowDown, OPTION);
    editing.key(Key::ArrowDown, OPTION);
    assert_eq!(editing.caret(), 7);
    editing.character_key('e', SHIFT.union(CONTROL));
    editing.character_key('a', SHIFT.union(CONTROL));
    assert_eq!(editing.selection(), (7, 4));
}

#[test]
fn the_delete_keys_take_a_character_a_word_or_the_rest_of_the_line() {
    let mut editing = Editing::field(&format!("one two{FAMILY}"));

    editing.key(Key::Backspace, NONE);
    assert_eq!(editing.text(), "one two", "a grapheme cluster is deleted whole");
    editing.key(Key::Backspace, OPTION);
    assert_eq!(editing.text(), "one ");
    editing.put_caret(0);
    editing.key(Key::Delete, NONE);
    assert_eq!(editing.text(), "ne ");
    editing.character_key('d', CONTROL);
    assert_eq!(editing.text(), "e ");
    editing.key(Key::Delete, OPTION);
    assert_eq!(editing.text(), " ");

    let mut line = Editing::area("abc def\nxyz");
    line.put_caret(5);
    line.key(Key::Backspace, COMMAND);
    assert_eq!(line.text(), "ef\nxyz");
    line.put_caret(1);
    line.character_key('k', CONTROL);
    assert_eq!(line.text(), "e\nxyz");
    line.character_key('h', CONTROL);
    assert_eq!((line.text(), line.caret()), ("\nxyz", 0));
    assert!(
        line.key(Key::Backspace, NONE),
        "a deletion that finds nothing still is the key of the text"
    );
    assert_eq!(line.text(), "\nxyz");
}

#[test]
fn a_deletion_takes_the_selection_whatever_it_would_pass_over() {
    let mut editing = Editing::field("one two three");
    editing.select(4, 7);

    editing.key(Key::Backspace, OPTION);

    assert_eq!((editing.text(), editing.caret()), ("one  three", 4));
}

#[test]
fn typing_is_undone_a_run_at_a_time() {
    let mut editing = Editing::field("");

    editing.type_text("hello world");
    assert!(editing.character_key('z', COMMAND));
    assert_eq!(
        editing.text(),
        "",
        "words typed without a pause between them are one run"
    );
    assert!(editing.character_key('z', SHIFT.union(COMMAND)));
    assert_eq!((editing.text(), editing.caret()), ("hello world", 11));

    editing.key(Key::ArrowLeft, NONE);
    editing.type_text("!!");
    editing.character_key('z', COMMAND);
    assert_eq!(
        (editing.text(), editing.caret()),
        ("hello world", 10),
        "a caret that moved begins a new run"
    );
    editing.character_key('z', COMMAND);
    assert_eq!(editing.text(), "");
}

#[test]
fn deletions_in_one_direction_are_one_step_and_a_change_of_kind_begins_another() {
    let mut editing = Editing::field("abcdef");

    editing.key(Key::Backspace, NONE);
    editing.key(Key::Backspace, NONE);
    editing.type_text("xy");
    editing.put_caret(0);
    editing.key(Key::Delete, NONE);
    editing.key(Key::Delete, NONE);
    assert_eq!(editing.text(), "cdxy");

    editing.character_key('z', COMMAND);
    assert_eq!((editing.text(), editing.caret()), ("abcdxy", 0));
    editing.character_key('z', COMMAND);
    assert_eq!((editing.text(), editing.caret()), ("abcd", 4));
    editing.character_key('z', COMMAND);
    assert_eq!((editing.text(), editing.caret()), ("abcdef", 6));
    assert!(
        editing.character_key('z', COMMAND),
        "undo with nothing to undo still is the key of the text"
    );
    assert_eq!(editing.text(), "abcdef");
}

#[test]
fn a_change_after_an_undo_leaves_nothing_to_redo_and_undo_gives_the_selection_back() {
    let mut editing = Editing::field("one two");
    editing.select(4, 7);

    editing.type_text("x");
    editing.character_key('z', COMMAND);
    assert_eq!((editing.text(), editing.selection()), ("one two", (4, 7)));

    editing.type_text("y");
    editing.character_key('z', SHIFT.union(COMMAND));
    assert_eq!(editing.text(), "one y", "the change that was undone is gone");
}

#[test]
fn the_clipboard_takes_what_is_cut_or_copied_and_a_paste_is_one_step() {
    let mut editing = Editing::field("one two");
    editing.select(4, 7);

    assert!(editing.character_key('c', COMMAND));
    assert_eq!(editing.clipboard.text().as_deref(), Some("two"));
    editing.put_caret(0);
    assert!(editing.character_key('v', COMMAND));
    assert_eq!((editing.text(), editing.caret()), ("twoone two", 3));

    editing.select(3, 6);
    assert!(editing.character_key('x', COMMAND));
    assert_eq!(editing.text(), "two two");
    assert_eq!(editing.clipboard.text().as_deref(), Some("one"));

    editing.character_key('z', COMMAND);
    editing.character_key('z', COMMAND);
    assert_eq!(editing.text(), "one two");
}

#[test]
fn one_line_takes_a_pasted_text_without_its_line_breaks() {
    let mut field = Editing::field("");
    field.clipboard.set_text("one\r\ntwo\nthree\n");
    field.character_key('v', COMMAND);
    assert_eq!(field.text(), "one two three");

    let mut area = Editing::area("");
    area.clipboard.set_text("one\r\ntwo\n");
    area.character_key('v', COMMAND);
    assert_eq!(area.text(), "one\ntwo\n");
}

#[test]
fn a_tree_without_a_clipboard_leaves_the_keys_of_the_clipboard_to_others() {
    let mut scene = Scene::new(RULES);
    let input = scene.add(scene.root(), Role::TextInput, &["field"]);
    scene.tree.set_focusability(input, Focusability::TabStop);
    scene.tree.set_text(input, "one");
    scene.tree.focus(input);
    scene.frame();
    let mut log = InputLog::default();

    assert!(!scene.press_key(Key::Character('v'), COMMAND, &mut log));
    assert!(!scene.press_key(Key::Character('c'), COMMAND, &mut log));
    assert_eq!(scene.tree.text(input), "one");
}

#[test]
fn transposing_swaps_the_characters_around_the_caret() {
    let mut editing = Editing::field("abcd");
    editing.put_caret(1);

    editing.character_key('t', CONTROL);
    assert_eq!((editing.text(), editing.caret()), ("bacd", 2));

    editing.put_caret(4);
    editing.character_key('t', CONTROL);
    assert_eq!(
        (editing.text(), editing.caret()),
        ("badc", 4),
        "at the end of a line the two characters before the caret change places"
    );
}

#[test]
fn return_breaks_a_line_of_an_area_and_is_left_to_others_by_a_field() {
    let mut area = Editing::area("ab");
    area.put_caret(1);
    assert!(area.key(Key::Enter, NONE));
    assert_eq!((area.text(), area.caret()), ("a\nb", 2));
    area.character_key('o', CONTROL);
    assert_eq!(
        (area.text(), area.caret()),
        ("a\n\nb", 2),
        "Control-O opens a line behind the caret"
    );

    let mut field = Editing::field("ab");
    assert!(!field.key(Key::Enter, NONE));
    assert!(
        !field.key(Key::Tab, NONE),
        "Tab moves focus and there is nowhere to move it"
    );
    assert_eq!(field.text(), "ab");
    assert_eq!(
        field.log.key_receivers(),
        [field.input, field.scene.root(), field.input, field.scene.root()],
        "a key the text does not take goes on to the elements around it"
    );
}

#[test]
fn an_element_hears_of_every_change_the_user_makes_to_its_text() {
    let mut editing = Editing::field("ab");

    editing.type_text("c");
    editing.key(Key::Backspace, NONE);
    editing.key(Key::ArrowLeft, NONE);
    editing.scene.tree.set_text(editing.input, "from outside");

    assert_eq!(
        editing.log.text_changes,
        [
            (editing.input, String::from("abc")),
            (editing.input, String::from("ab"))
        ],
        "a caret that moves and a text set from outside are no changes the user made"
    );
    assert_eq!(editing.caret(), 12, "a text set from outside has its caret at the end");
}

#[test]
fn an_element_that_uses_a_key_keeps_it_from_its_text() {
    let mut editing = Editing::field("ab");
    editing.log.consumers.push(editing.input);

    assert!(editing.key(Key::Backspace, NONE));

    assert_eq!(editing.text(), "ab");
}

#[test]
fn the_lines_of_an_area_wrap_at_the_width_of_its_box() {
    let mut editing = Editing::of(Role::TextArea, "growing", "aaaa bbbb");
    assert_eq!(
        editing.scene.border_box(editing.input).height.to_bits(),
        LINE_HEIGHT.to_bits()
    );

    editing.type_text(" cc");

    assert_eq!(
        editing
            .drawn_lines()
            .iter()
            .map(|(_, line, _)| line.as_str())
            .collect::<Vec<_>>(),
        ["aaaa bbbb ", "cc"]
    );
    assert_eq!(editing.shown_caret(), caret_at(2, 1));
    assert_eq!(
        editing.scene.border_box(editing.input).height.to_bits(),
        (2.0 * LINE_HEIGHT).to_bits(),
        "an area without a height of its own is as high as its lines"
    );

    editing.key(Key::Enter, NONE);
    assert_eq!(
        editing.shown_caret(),
        caret_at(0, 2),
        "the caret stands on the empty line behind a line break"
    );
    assert_eq!(
        editing.scene.border_box(editing.input).height.to_bits(),
        (3.0 * LINE_HEIGHT).to_bits()
    );
}

#[test]
fn a_field_scrolls_its_line_to_keep_the_caret_in_view() {
    let mut editing = Editing::field("0123456789abcdefghij");
    assert_eq!(
        editing.scroll(),
        (101.0, 0.0),
        "the caret behind the last character stands at the right edge of the box"
    );
    assert_eq!(editing.shown_caret(), Rect::new(99.0, 0.0, 1.0, LINE_HEIGHT));

    editing.key(Key::ArrowLeft, NONE);
    assert_eq!(editing.scroll(), (101.0, 0.0), "a caret in view scrolls nothing");
    editing.key(Key::ArrowLeft, COMMAND);
    assert_eq!(editing.scroll(), (0.0, 0.0));
    editing.put_caret(15);
    assert_eq!(
        editing.scroll(),
        (51.0, 0.0),
        "the line moves the least that shows the caret"
    );

    editing.key(Key::ArrowRight, COMMAND);
    for _ in 0..8 {
        editing.key(Key::Backspace, NONE);
    }
    assert_eq!(
        editing.scroll(),
        (21.0, 0.0),
        "a line that got shorter moves back into the box"
    );
}

#[test]
fn an_area_scrolls_its_lines_to_keep_the_caret_in_view() {
    let mut editing = Editing::area("one\ntwo\nthree\nfour\nfive");
    assert_eq!(editing.scroll(), (0.0, 60.0), "five lines in a box of two");

    editing.key(Key::ArrowUp, NONE);
    assert_eq!(editing.scroll(), (0.0, 60.0));
    editing.key(Key::ArrowUp, NONE);
    assert_eq!(editing.scroll(), (0.0, 40.0));
    editing.key(Key::ArrowUp, COMMAND);
    assert_eq!(editing.scroll(), (0.0, 0.0));
    assert_eq!(
        editing
            .drawn_lines()
            .iter()
            .map(|(_, line, _)| line.as_str())
            .collect::<Vec<_>>(),
        ["one", "two"],
        "only the lines that show are drawn"
    );
}

#[test]
fn home_and_end_scroll_and_leave_the_caret() {
    let mut editing = Editing::area("one\ntwo\nthree\nfour\nfive");

    assert!(editing.key(Key::Home, NONE));
    assert_eq!((editing.scroll(), editing.caret()), ((0.0, 0.0), 23));
    editing.key(Key::PageDown, NONE);
    assert_eq!((editing.scroll(), editing.caret()), ((0.0, 40.0), 23));
    editing.key(Key::End, NONE);
    assert_eq!(editing.scroll(), (0.0, 60.0));
    editing.key(Key::PageUp, OPTION);
    assert_eq!(
        editing.caret(),
        12,
        "Option moves the caret up by the two lines the box shows"
    );
}

#[test]
fn a_press_puts_the_caret_at_the_nearest_place_and_shift_extends_the_selection() {
    let mut editing = Editing::field("one two");

    editing.press(34.0, 10.0, 1, NONE);
    editing.release(34.0, 10.0);
    assert_eq!(editing.caret(), 3);
    editing.press(36.0, 10.0, 1, NONE);
    editing.release(36.0, 10.0);
    assert_eq!(editing.caret(), 4);
    editing.press(95.0, 10.0, 1, NONE);
    editing.release(95.0, 10.0);
    assert_eq!(
        editing.caret(),
        7,
        "a press past the end of the line is a press at its end"
    );

    editing.press(10.0, 10.0, 1, SHIFT);
    editing.release(10.0, 10.0);
    assert_eq!(editing.selection(), (7, 1));
}

#[test]
fn the_pointer_selects_from_its_press_to_where_it_is_dragged() {
    let mut editing = Editing::area("one two\nsix ten");

    editing.press(10.0, 10.0, 1, NONE);
    editing.pointer(PointerInputKind::Moved, 30.0, 30.0, NONE);
    assert_eq!(editing.selection(), (1, 11));
    editing.pointer(PointerInputKind::Moved, 300.0, -40.0, NONE);
    assert_eq!(
        editing.selection(),
        (1, 7),
        "the pointer selects on while it is outside the box"
    );
    editing.release(300.0, -40.0);
    editing.pointer(PointerInputKind::Moved, 10.0, 30.0, NONE);
    assert_eq!(editing.selection(), (1, 7), "a released pointer selects no more");
}

#[test]
fn two_presses_select_a_word_and_three_a_paragraph() {
    let mut editing = Editing::area("one two\nsix ten");

    editing.press(55.0, 10.0, 2, NONE);
    assert_eq!(editing.selection(), (4, 7));
    editing.pointer(PointerInputKind::Moved, 10.0, 30.0, NONE);
    assert_eq!(
        editing.selection(),
        (4, 11),
        "a drag after two presses selects word by word"
    );
    editing.pointer(PointerInputKind::Moved, 10.0, 10.0, NONE);
    assert_eq!(editing.selection(), (7, 0));
    editing.release(10.0, 10.0);

    editing.press(15.0, 30.0, 3, NONE);
    assert_eq!(editing.selection(), (8, 15));
    editing.release(15.0, 30.0);
}

#[test]
fn a_caret_is_as_high_as_the_font_and_stands_on_the_baseline_of_a_taller_line() {
    let mut editing = Editing::of(Role::TextArea, "leading", "ab\ncd");

    editing.put_caret(4);

    // A line of 1.45 times the font size of 16 pixels leaves 3.2 pixels of leading over the
    // 20 pixels of the font, the half above the content area cut to a whole pixel. The caret
    // starts that far below the top of its line, on whole device pixels.
    let line_height = (16.0_f32 * 1.45 * 64.0).trunc() / 64.0;
    let half_leading = ((line_height - LINE_HEIGHT) / 2.0).floor();
    assert_eq!(
        editing.shown_caret(),
        Rect::new(10.0, (line_height + half_leading).round(), 1.0, LINE_HEIGHT)
    );
}

#[test]
fn a_caret_blinks_and_is_shown_again_as_soon_as_it_moves() {
    let mut editing = Editing::field("ab");
    let milliseconds = Duration::from_millis;
    assert_eq!(editing.shown_caret(), caret_at(2, 0));

    editing.scene.frame_at(milliseconds(529));
    assert_eq!(editing.fills().len(), 1);
    editing.scene.frame_at(milliseconds(530));
    assert!(
        editing.fills().is_empty(),
        "the caret is hidden for the second half of its period"
    );
    editing.scene.frame_at(milliseconds(1060));
    assert_eq!(editing.fills().len(), 1);

    editing.scene.frame_at(milliseconds(1700));
    assert!(editing.fills().is_empty());
    editing.scene.press_key(Key::ArrowLeft, NONE, &mut editing.log);
    editing.scene.frame_at(milliseconds(1710));
    assert_eq!(editing.shown_caret(), caret_at(1, 0));
    editing.scene.frame_at(milliseconds(2230));
    assert_eq!(
        editing.fills().len(),
        1,
        "a whole half period from the frame that showed it moved"
    );
}

#[test]
fn the_caret_and_the_selection_are_shown_only_while_the_text_has_focus() {
    let mut editing = Editing::field("one two");
    editing.select(0, 3);
    let selection_color = editing.scene.light_color("color-selection");
    assert_eq!(
        editing.fills(),
        [(Rect::new(0.0, 0.0, 30.0, LINE_HEIGHT), selection_color)],
        "the highlight covers the line box of the selected text; the caret is hidden"
    );

    editing.scene.tree.blur();
    editing.scene.frame();
    assert!(editing.fills().is_empty());
    assert_eq!(editing.drawn_lines().len(), 1, "the text itself stays");

    editing.scene.tree.focus(editing.input);
    editing.scene.frame();
    assert_eq!(editing.fills().len(), 1);
    editing.put_caret(1);
    assert_eq!(
        editing.fills(),
        [(caret_at(1, 0), editing.scene.light_color("color-accent"))],
        "the caret has the color `caret-color` gives it"
    );
}

#[test]
fn a_selection_over_several_lines_is_highlighted_line_by_line() {
    let mut editing = Editing::area("one two\nsix ten");

    editing.select(4, 11);

    let boxes: Vec<Rect> = editing.fills().iter().map(|(rect, _)| *rect).collect();
    assert_eq!(
        boxes,
        [
            Rect::new(40.0, 0.0, 30.0, LINE_HEIGHT),
            Rect::new(0.0, LINE_HEIGHT, 30.0, LINE_HEIGHT),
        ]
    );
}

#[test]
fn an_empty_text_shows_its_placeholder_in_the_color_of_the_pseudo_element() {
    let mut editing = Editing::field("");
    editing.scene.tree.set_placeholder(editing.input, "Folder name");
    editing.scene.frame();

    let baseline = Point { x: 0.0, y: 15.0 };
    assert_eq!(
        editing.drawn_lines(),
        [(
            baseline,
            String::from("Folder name"),
            editing.scene.light_color("color-text-3")
        )]
    );
    assert_eq!(
        editing.shown_caret(),
        caret_at(0, 0),
        "the caret stands before the placeholder"
    );

    editing.type_text("a");
    assert_eq!(
        editing.drawn_lines(),
        [(baseline, String::from("a"), editing.scene.light_color("color-text"))],
        "a text with characters hides it"
    );
}

#[test]
fn a_text_being_composed_is_underlined_until_it_is_settled() {
    let mut editing = Editing::field("ab");
    editing.put_caret(1);
    let underlines = |editing: &Editing| -> Vec<Rect> {
        editing
            .scene
            .tree
            .display_list()
            .items()
            .iter()
            .filter_map(|item| match item {
                DisplayItem::TextDecoration { rect, .. } => Some(*rect),
                _ => None,
            })
            .collect()
    };

    editing
        .scene
        .tree
        .set_marked_text(editing.input, "xyz", 3..3, None, &mut editing.log);
    editing.scene.frame();
    assert_eq!(editing.text(), "axyzb");
    assert_eq!(editing.scene.tree.marked_text(editing.input), Some(1..4));
    assert_eq!(underlines(&editing), [Rect::new(10.0, 17.0, 30.0, 1.0)]);
    assert_eq!(
        editing.scene.tree.text_range_box(editing.input, 1..4),
        Rect::new(10.0, 0.0, 30.0, LINE_HEIGHT),
        "the window of candidates goes next to the composed text"
    );

    editing
        .scene
        .tree
        .insert_text(editing.input, "\u{6771}", None, &mut editing.log);
    editing.scene.frame();
    assert_eq!((editing.text(), editing.caret()), ("a\u{6771}b", 4));
    assert_eq!(editing.scene.tree.marked_text(editing.input), None);
    assert!(underlines(&editing).is_empty());
}

#[test]
fn a_command_by_lines_given_before_the_text_is_laid_out_again_goes_by_paragraphs() {
    let mut editing = Editing::area("abcdef\nxy");

    editing
        .scene
        .tree
        .insert_text(editing.input, "z", None, &mut editing.log);
    editing.scene.press_key(Key::ArrowUp, NONE, &mut editing.log);

    assert_eq!(
        editing.caret(),
        3,
        "three characters into its paragraph, the caret goes three characters into the one above"
    );
}

#[test]
fn a_place_inside_a_grapheme_cluster_falls_back_on_its_start() {
    let mut editing = Editing::field(&format!("a{FAMILY}b"));

    editing.select(3, 1 + FAMILY.len() + 1);

    assert_eq!(editing.selection(), (1, 1 + FAMILY.len() + 1));
    assert_eq!(
        editing
            .scene
            .tree
            .text_offset_at(editing.input, Point { x: 12.0, y: 5.0 }),
        1
    );
}

/// The node of the update with the ID, if the update holds it.
fn node_of(update: &TreeUpdate, id: NodeId) -> Option<&morok_core::accesskit::Node> {
    update.nodes.iter().find(|(node, _)| *node == id).map(|(_, node)| node)
}

#[test]
fn an_assistive_technology_reads_the_lines_and_the_selection_of_an_edited_text() {
    let mut editing = Editing::area("one two\nsix");
    editing
        .scene
        .tree
        .set_accessible_role(editing.input, Some(AccessibleRole::MultilineTextInput));
    editing.scene.tree.set_placeholder(editing.input, "Note");
    editing.select(4, 9);
    let input_node = NodeId::from(editing.input);

    let update = editing.accessibility_update();

    let input = node_of(&update, input_node).expect("the update holds the node of the text");
    assert_eq!(input.role(), AccessibleRole::MultilineTextInput);
    assert_eq!(input.value(), Some("one two\nsix"));
    assert_eq!(input.placeholder(), Some("Note"));
    assert!(input.supports_action(Action::SetTextSelection));
    assert!(input.supports_action(Action::ReplaceSelectedText));
    let runs = input.children();
    assert_eq!(runs.len(), 2);
    let lines: Vec<_> = runs
        .iter()
        .map(|run| node_of(&update, *run).expect("the update holds the node of every line"))
        .collect();
    assert_eq!(lines[0].role(), AccessibleRole::TextRun);
    assert_eq!(
        (lines[0].value(), lines[1].value()),
        (Some("one two\n"), Some("six")),
        "a line holds the line break it ends with"
    );
    assert_eq!(lines[0].character_lengths(), [1; 8]);
    assert_eq!(lines[0].word_starts(), [0, 4]);
    assert_eq!(
        lines[1].character_positions(),
        Some(&[0.0, CHARACTER_WIDTH, 2.0 * CHARACTER_WIDTH][..])
    );
    assert_eq!(
        input.text_selection(),
        Some(&NodeSelection {
            anchor: TextPosition {
                node: runs[0],
                character_index: 4
            },
            focus: TextPosition {
                node: runs[1],
                character_index: 1
            },
        })
    );
}

#[test]
fn an_assistive_technology_moves_the_selection_and_changes_the_text() {
    let mut editing = Editing::area("one two\nsix");
    editing
        .scene
        .tree
        .set_accessible_role(editing.input, Some(AccessibleRole::MultilineTextInput));
    let input_node = NodeId::from(editing.input);
    let update = editing.accessibility_update();
    let runs = node_of(&update, input_node)
        .expect("the update holds the node of the text")
        .children()
        .to_vec();
    let request = |action: Action, data: ActionData| ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: input_node,
        data: Some(data),
    };

    let asked = NodeSelection {
        anchor: TextPosition {
            node: runs[0],
            character_index: 4,
        },
        focus: TextPosition {
            node: runs[1],
            character_index: 3,
        },
    };
    editing.scene.tree.accessibility_action(
        &request(Action::SetTextSelection, ActionData::SetTextSelection(asked)),
        &mut editing.log,
    );
    assert_eq!(editing.selection(), (4, 11));

    editing.scene.tree.accessibility_action(
        &request(Action::ReplaceSelectedText, ActionData::Value("2".into())),
        &mut editing.log,
    );
    assert_eq!((editing.text(), editing.caret()), ("one 2", 5));

    editing.scene.tree.accessibility_action(
        &request(Action::SetValue, ActionData::Value("all new".into())),
        &mut editing.log,
    );
    assert_eq!(editing.text(), "all new");
    assert_eq!(
        editing.log.text_changes.last(),
        Some(&(editing.input, String::from("all new"))),
        "the element hears of a change an assistive technology made as of one the user made"
    );
}

#[test]
fn a_key_that_goes_up_edits_nothing() {
    let mut editing = Editing::field("ab");
    let released = KeyEvent {
        phase: KeyPhase::Released,
        key: Key::Backspace,
        modifiers: NONE,
        is_repeat: false,
    };

    assert!(!editing.scene.tree.key_input(&released, &mut editing.log));

    assert_eq!(editing.text(), "ab");
}
