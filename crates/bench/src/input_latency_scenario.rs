use std::time::{Duration, Instant};

use morok::core::{DisplayItem, DisplayList, ElementTree, Role};
use morok::{Application, Window};
use morok_catalog::{Catalog, Page, WINDOW_TITLE};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSEvent, NSEventModifierFlags, NSEventType, NSWindow};
use objc2_foundation::{NSPoint, NSString};

use crate::bench_error::BenchError;
use crate::input_latency_budget::InputLatencyBudget;
use crate::input_latency_gate;
use crate::verdict::Verdict;

/// How long the window gets to present its first frame, or to show the glyph of a keypress.
const FRAME_TIMEOUT: Duration = Duration::from_secs(5);
/// The event loop is given this long at a time while a frame is waited for: far less than a
/// frame, so that a poll sees one presented frame at a time.
const FRAME_POLL: Duration = Duration::from_millis(1);
/// The keys typed, by their virtual key code and the character they type: letters the font
/// sets as one glyph each, never joined into a ligature.
const TYPED_KEYS: [(u16, &str); 5] = [(0x00, "a"), (0x0B, "b"), (0x08, "c"), (0x02, "d"), (0x0E, "e")];

/// The placeholder of the field of the catalog the keys are typed into.
const FIELD_PLACEHOLDER: &str = "Name";

/// Types into the first text field of the page of the text fields of the catalog, a key at a
/// time, and counts for each key the frames run after it before the first that shows its glyph.
/// A key goes through the input methods of the system as a key of the keyboard does. The
/// window runs a frame at every refresh of the display, as a window with something in motion
/// does, so the frame of a keypress is the first that runs after it.
pub fn run(scenario: &str, budget: &InputLatencyBudget) -> Result<Verdict, BenchError> {
    let main_thread = MainThreadMarker::new().expect("the harness runs on the main thread");
    let catalog = Catalog::launch(Page::TextFields)?;
    let application = catalog.application();
    let windows = catalog.windows();
    let window = &windows[0];
    wait_for_first_frame(application, window)?;
    let focused = window.with_tree(|tree| {
        let field = text_input_with_placeholder(tree, FIELD_PLACEHOLDER).expect("the page shows the field of the name");
        tree.focus(field)
    });
    assert!(focused, "the line of the field takes focus");
    let native_window = NSApplication::sharedApplication(main_thread)
        .windows()
        .to_vec()
        .into_iter()
        .find(|native| native.isVisible() && native.title().to_string() == WINDOW_TITLE)
        .expect("the catalog has opened its window");
    let mut keys = TYPED_KEYS.iter().cycle();
    let mut type_key = || {
        let (key_code, character) = keys.next().expect("the typed keys cycle");
        late_frames_of_keypress(application, window, &native_window, *key_code, character)
    };
    for _ in 0..budget.warm_up_keypresses {
        type_key()?;
    }
    let late_frames = (0..budget.measured_keypresses)
        .map(|_| type_key())
        .collect::<Result<Vec<_>, _>>()?;
    Ok(input_latency_gate::judge(scenario, &late_frames, budget))
}

/// Presses a key and counts the frames run after it before the first that draws as many glyphs
/// as the focused field holds characters.
fn late_frames_of_keypress(
    application: &Application,
    window: &Window,
    native_window: &Retained<NSWindow>,
    key_code: u16,
    character: &str,
) -> Result<usize, BenchError> {
    let run_before = window.frame_statistics().display_link_callbacks;
    for kind in [NSEventType::KeyDown, NSEventType::KeyUp] {
        native_window.sendEvent(&key_event(native_window, kind, key_code, character));
    }
    let pressed = Instant::now();
    let mut run = run_before;
    loop {
        run = wait_for_next_frame(application, window, run)?;
        if window.with_tree(|tree| shows_every_character(tree)) {
            let late = run - run_before - 1;
            return Ok(usize::try_from(late).expect("the frames of one keypress are few"));
        }
        if pressed.elapsed() >= FRAME_TIMEOUT {
            return Err(BenchError::NoGlyph { waited: FRAME_TIMEOUT });
        }
    }
}

fn wait_for_first_frame(application: &Application, window: &Window) -> Result<(), BenchError> {
    let started = Instant::now();
    while window.frame_statistics().presented_frames == 0 {
        if started.elapsed() >= FRAME_TIMEOUT {
            return Err(BenchError::NoFirstFrame { waited: FRAME_TIMEOUT });
        }
        application.run_for(FRAME_POLL);
    }
    Ok(())
}

/// Runs the event loop, asking for a frame all the while, until a frame after the `run`-th
/// has run; returns how many have run then.
fn wait_for_next_frame(application: &Application, window: &Window, run: u64) -> Result<u64, BenchError> {
    let started = Instant::now();
    loop {
        window.with_tree(ElementTree::request_frame);
        let now_run = window.frame_statistics().display_link_callbacks;
        if now_run > run {
            return Ok(now_run);
        }
        if started.elapsed() >= FRAME_TIMEOUT {
            return Err(BenchError::NoFrame { waited: FRAME_TIMEOUT });
        }
        application.run_for(FRAME_POLL);
    }
}

/// The line of text the user edits that shows `placeholder` while it is empty.
fn text_input_with_placeholder(tree: &ElementTree, placeholder: &str) -> Option<morok::core::ElementId> {
    let mut unvisited = vec![tree.root()];
    while let Some(element) = unvisited.pop() {
        if tree.role(element) == Role::TextInput && tree.placeholder(element) == placeholder {
            return Some(element);
        }
        unvisited.extend(tree.children(element));
    }
    None
}

/// Whether the last display list draws a glyph for every character of the focused text, in
/// the part of the list that draws the focused text and nothing else.
fn shows_every_character(tree: &ElementTree) -> bool {
    let Some(focused) = tree.focused_text_input() else {
        return false;
    };
    let mut display_list = DisplayList::default();
    tree.subtree_display_list(focused, &mut display_list);
    let drawn: usize = display_list
        .items()
        .iter()
        .filter_map(|item| match item {
            DisplayItem::GlyphRun { glyphs, .. } => Some(display_list.glyphs(*glyphs).len()),
            _ => None,
        })
        .sum();
    drawn == tree.text(focused).chars().count()
}

fn key_event(window: &NSWindow, kind: NSEventType, key_code: u16, character: &str) -> Retained<NSEvent> {
    let characters = NSString::from_str(character);
    NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
        kind,
        NSPoint::ZERO,
        NSEventModifierFlags::empty(),
        0.0,
        window.windowNumber(),
        None,
        &characters,
        &characters,
        false,
        key_code,
    )
    .expect("the key event is made")
}
