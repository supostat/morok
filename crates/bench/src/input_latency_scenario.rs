use std::time::{Duration, Instant};

use morok::components::TextField;
use morok::core::{DisplayItem, ElementTree, Size};
use morok::{Application, Styles, Window, WindowOptions};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSEvent, NSEventModifierFlags, NSEventType, NSWindow};
use objc2_foundation::{NSPoint, NSString};

use crate::bench_error::BenchError;
use crate::input_latency_budget::InputLatencyBudget;
use crate::input_latency_gate;
use crate::verdict::Verdict;

const CONTENT_SIZE: Size = Size {
    width: 320.0,
    height: 80.0,
};
/// How long the window gets to present its first frame, or to show the glyph of a keypress.
const FRAME_TIMEOUT: Duration = Duration::from_secs(5);
/// The event loop is given this long at a time while a frame is waited for: far less than a
/// frame, so that a poll sees one presented frame at a time.
const FRAME_POLL: Duration = Duration::from_millis(1);
/// The keys typed, by their virtual key code and the character they type: letters the font
/// sets as one glyph each, never joined into a ligature.
const TYPED_KEYS: [(u16, &str); 5] = [(0x00, "a"), (0x0B, "b"), (0x08, "c"), (0x02, "d"), (0x0E, "e")];

/// Types into a text field of the kit in a window, a key at a time, and counts for each key
/// the frames run after it before the first that shows its glyph. A key goes through the input
/// methods of the system as a key of the keyboard does. The window runs a frame at every
/// refresh of the display, as a window with something in motion does, so the frame of a
/// keypress is the first that runs after it.
pub fn run(scenario: &str, budget: &InputLatencyBudget) -> Result<Verdict, BenchError> {
    let main_thread = MainThreadMarker::new().expect("the harness runs on the main thread");
    let application = Application::new(Styles::of_kit())?;
    let value = application.update(|runtime| runtime.signal(String::new()));
    let options = WindowOptions {
        title: String::from("morok: input latency"),
        content_size: CONTENT_SIZE,
    };
    let mut field = None;
    let window = application.open_window(&options, |mount| {
        let built = TextField::new(value).placeholder("Type here").mount(mount);
        field = Some(built);
        built
    });
    let field = field.expect("the window mounted its root component");
    wait_for_first_frame(&application, &window)?;
    let focused = window.with_tree(|tree| {
        let input = *tree.children(field).last().expect("the field holds its line");
        tree.focus(input)
    });
    assert!(focused, "the line of the field takes focus");
    let native_window = NSApplication::sharedApplication(main_thread)
        .windows()
        .firstObject()
        .expect("the application has opened its window");
    let mut keys = TYPED_KEYS.iter().cycle();
    let mut type_key = || {
        let (key_code, character) = keys.next().expect("the typed keys cycle");
        late_frames_of_keypress(&application, &window, &native_window, *key_code, character)
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

/// Whether the last display list draws a glyph for every character of the focused text.
fn shows_every_character(tree: &ElementTree) -> bool {
    let Some(focused) = tree.focused_text_input() else {
        return false;
    };
    let display_list = tree.display_list();
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
