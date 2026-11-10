mod support;

use std::process::ExitCode;
use std::ptr;
use std::time::Duration;

use morok_core::accesskit::{ActionRequest, TreeUpdate};
use morok_core::{
    AccessibilityAction, ElementId, ElementTree, Focusability, InputHandler, Key, KeyChord, KeyEvent, KeyPhase,
    Modifiers, Point, PointerButton, PointerEvent, PointerEventKind, PointerInput, PointerInputKind, Propagation,
    ScrollInput, ShortcutId, ShortcutScope, Size,
};
use morok_platform::{Application, SurfaceContent, Window, WindowOptions};
use morok_style::{ColorScheme, States};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSEvent, NSEventModifierFlags, NSEventType, NSView, NSWindow};
use objc2_foundation::{NSPoint, NSString};
use support::{Expectations, WindowScene, run_main_loop};

const PANEL: &str = ".panel { display: flex; gap: 20px; padding: 20px; background-color: var(--color-paper); }\n\
    .button { width: 100px; height: 30px; background-color: var(--color-blue); }\n\
    .button:hover { background-color: var(--color-red); }";

const CONTENT_SIZE: Size = Size {
    width: 400.0,
    height: 300.0,
};
/// A point of the surface over each of the two buttons.
const OVER_FIRST: Point = Point { x: 60.0, y: 35.0 };
const OVER_SECOND: Point = Point { x: 180.0, y: 35.0 };

/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);

/// What the tree routed to its elements.
#[derive(Debug, Default)]
struct Heard {
    pointer_events: Vec<(ElementId, PointerEvent)>,
    key_events: Vec<(ElementId, KeyEvent)>,
    shortcuts: Vec<ShortcutId>,
    /// The keys the root uses.
    keys_of_the_root: Vec<Key>,
    root: Option<ElementId>,
}

impl InputHandler for Heard {
    fn pointer_event(&mut self, element: ElementId, event: &PointerEvent) -> Propagation {
        self.pointer_events.push((element, *event));
        Propagation::Continue
    }

    fn key_event(&mut self, element: ElementId, event: &KeyEvent) -> Propagation {
        self.key_events.push((element, *event));
        if Some(element) == self.root && self.keys_of_the_root.contains(&event.key) {
            Propagation::Stop
        } else {
            Propagation::Continue
        }
    }

    fn shortcut(&mut self, shortcut: ShortcutId) {
        self.shortcuts.push(shortcut);
    }

    fn accessibility_action(&mut self, _element: ElementId, _action: &AccessibilityAction<'_>) {}

    fn text_changed(&mut self, _element: ElementId, _text: &str) {}
}

/// A scene in a window that gives the input of the window to its tree and remembers both what
/// the window reported and what the tree made of it.
struct InputScene {
    window_scene: WindowScene,
    pointer_inputs: Vec<PointerInput>,
    key_inputs: Vec<KeyEvent>,
    heard: Heard,
}

impl SurfaceContent for InputScene {
    fn frame(&mut self, viewport: Size, scale: f32, now: Duration, show: &mut dyn FnMut(&ElementTree)) {
        self.window_scene.scene.tree.refresh_hover(&mut self.heard);
        self.window_scene.frame(viewport, scale, now, show);
    }

    fn color_scheme_changed(&mut self, color_scheme: ColorScheme) {
        self.window_scene.color_scheme_changed(color_scheme);
    }

    fn reduced_motion_changed(&mut self, reduces_motion: bool) {
        self.window_scene.reduced_motion_changed(reduces_motion);
    }

    fn pointer_input(&mut self, input: &PointerInput) {
        self.pointer_inputs.push(*input);
        self.window_scene.scene.tree.pointer_input(input, &mut self.heard);
    }

    fn scroll_input(&mut self, _input: &ScrollInput) {}

    fn key_input(&mut self, event: &KeyEvent) -> bool {
        self.key_inputs.push(*event);
        self.window_scene.scene.tree.key_input(event, &mut self.heard)
    }

    fn accessibility_tree(&mut self) -> TreeUpdate {
        self.window_scene.accessibility_tree()
    }

    fn accessibility_update(&mut self) -> TreeUpdate {
        self.window_scene.accessibility_update()
    }

    fn accessibility_action(&mut self, request: &ActionRequest) {
        self.window_scene
            .scene
            .tree
            .accessibility_action(request, &mut self.heard);
    }
}

/// Makes the events `AppKit` would send for a mouse and a keyboard and gives them to a window.
///
/// Presses, releases, drags and keys go through the window, which finds the view under the
/// point or its first responder as it does for the events of the system. A move and an exit
/// are given to the view itself: `AppKit` sends those to the owner of a tracking area by where
/// the pointer of the system is, and the test does not move that pointer.
struct EventSource {
    window: Retained<NSWindow>,
    surface_view: Retained<NSView>,
}

impl EventSource {
    /// The only window of the application.
    fn for_the_window(main_thread: MainThreadMarker) -> Self {
        let window = NSApplication::sharedApplication(main_thread)
            .windows()
            .firstObject()
            .expect("the application has opened its window");
        let surface_view = window.contentView().expect("the window shows its surface");
        Self { window, surface_view }
    }

    fn location(at: Point) -> NSPoint {
        NSPoint::new(f64::from(at.x), f64::from(CONTENT_SIZE.height - at.y))
    }

    fn mouse_event(
        &self,
        kind: NSEventType,
        at: Point,
        flags: NSEventModifierFlags,
        click_count: isize,
    ) -> Retained<NSEvent> {
        NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
            kind,
            Self::location(at),
            flags,
            0.0,
            self.window.windowNumber(),
            None,
            0,
            click_count,
            1.0,
        )
        .expect("the mouse event is made")
    }

    /// The pointer moves over the surface with no button down.
    fn mouse_moved(&self, at: Point, flags: NSEventModifierFlags) {
        self.surface_view
            .mouseMoved(&self.mouse_event(NSEventType::MouseMoved, at, flags, 0));
    }

    /// The pointer leaves the surface.
    fn mouse_exited(&self, at: Point) {
        // SAFETY: the event carries no user data, which a null pointer says.
        let event = unsafe {
            NSEvent::enterExitEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_trackingNumber_userData(
                NSEventType::MouseExited,
                Self::location(at),
                NSEventModifierFlags::empty(),
                0.0,
                self.window.windowNumber(),
                None,
                0,
                0,
                ptr::null_mut(),
            )
        }
        .expect("the exit event is made");
        self.surface_view.mouseExited(&event);
    }

    /// A press, a release or a drag at a point of the surface.
    fn mouse(&self, kind: NSEventType, at: Point, flags: NSEventModifierFlags, click_count: isize) {
        self.window.sendEvent(&self.mouse_event(kind, at, flags, click_count));
    }

    /// A key event: `characters` is what the key types with no modifier but Shift held.
    fn key(&self, kind: NSEventType, key_code: u16, characters: &str, flags: NSEventModifierFlags, is_repeat: bool) {
        let characters = NSString::from_str(characters);
        let event = NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
            kind,
            NSPoint::ZERO,
            flags,
            0.0,
            self.window.windowNumber(),
            None,
            &characters,
            &characters,
            is_repeat,
            key_code,
        )
        .expect("the key event is made");
        self.window.sendEvent(&event);
    }

    fn key_down(&self, key_code: u16, characters: &str, flags: NSEventModifierFlags) {
        self.key(NSEventType::KeyDown, key_code, characters, flags, false);
    }
}

/// The keys that are told by their place on the keyboard: the virtual key code, what `AppKit`
/// says the key types, and the key the tree is to hear of.
const NAMED_KEYS: [(u16, &str, Key); 12] = [
    (0x24, "\r", Key::Enter),
    (0x4C, "\u{3}", Key::Enter),
    (0x31, " ", Key::Space),
    (0x35, "\u{1b}", Key::Escape),
    (0x33, "\u{7f}", Key::Backspace),
    (0x75, "\u{f728}", Key::Delete),
    (0x7B, "\u{f702}", Key::ArrowLeft),
    (0x7E, "\u{f700}", Key::ArrowUp),
    (0x73, "\u{f729}", Key::Home),
    (0x79, "\u{f72d}", Key::PageDown),
    (0x60, "\u{f708}", Key::Function(5)),
    (0x6F, "\u{f70f}", Key::Function(12)),
];

/// The window of the test with what the expectations read of it.
struct Stage {
    window: Window<InputScene>,
    source: EventSource,
    expectations: Expectations,
    root: ElementId,
    first: ElementId,
    second: ElementId,
    save: ShortcutId,
}

impl Stage {
    /// A window on a panel of two buttons, both tab stops, with a shortcut on Command-S.
    fn open(application: &Application, main_thread: MainThreadMarker) -> Self {
        let options = WindowOptions {
            title: String::from("morok: event translation"),
            content_size: CONTENT_SIZE,
        };
        let mut built = None;
        let window = Window::open(application, &options, |frame_scheduler, settings| {
            let mut window_scene = WindowScene::new(PANEL, frame_scheduler, settings);
            let scene = &mut window_scene.scene;
            let root = scene.root();
            scene.add_classes(root, &["panel"]);
            let first = scene.add(root, &["button"]);
            let second = scene.add(root, &["button"]);
            scene.tree.set_focusability(first, Focusability::TabStop);
            scene.tree.set_focusability(second, Focusability::TabStop);
            let save_chord = KeyChord {
                key: Key::Character('s'),
                modifiers: Modifiers::COMMAND,
            };
            let save = scene.tree.register_shortcut(ShortcutScope::Surface, save_chord);
            built = Some((root, first, second, save));
            let mut keys_of_the_root: Vec<Key> = NAMED_KEYS.iter().map(|(_, _, key)| *key).collect();
            keys_of_the_root.push(Key::Character('a'));
            InputScene {
                window_scene,
                pointer_inputs: Vec::new(),
                key_inputs: Vec::new(),
                heard: Heard {
                    keys_of_the_root,
                    root: Some(root),
                    ..Heard::default()
                },
            }
        });
        let (root, first, second, save) = built.expect("the window built its content");
        Self {
            window,
            source: EventSource::for_the_window(main_thread),
            expectations: Expectations::default(),
            root,
            first,
            second,
            save,
        }
    }

    fn states(&self, element: ElementId) -> States {
        self.window
            .update(|content| content.window_scene.scene.tree.states(element))
    }

    fn focused(&self) -> Option<ElementId> {
        self.window.update(|content| content.window_scene.scene.tree.focused())
    }

    fn last_pointer_input(&self) -> Option<PointerInput> {
        self.window.update(|content| content.pointer_inputs.last().copied())
    }

    fn last_key_input(&self) -> Option<KeyEvent> {
        self.window.update(|content| content.key_inputs.last().copied())
    }

    /// Whether the element heard of a click of that many presses.
    fn was_clicked(&self, element: ElementId, click_count: u32) -> bool {
        let clicked = PointerEventKind::Clicked { click_count };
        self.window.update(|content| {
            content
                .heard
                .pointer_events
                .iter()
                .any(|(heard_by, event)| *heard_by == element && event.kind == clicked)
        })
    }

    fn the_first_frame_lays_the_tree_out(&mut self) {
        run_main_loop(SETTLING);
        let statistics = self.window.frame_statistics();
        self.expectations.expect(
            statistics.presented_frames >= 1,
            "the window lays its tree out in a first frame; is the display on?",
            &format!("{statistics:?}"),
        );
    }

    fn a_move_hovers_what_is_under_the_pointer(&mut self) {
        self.source.mouse_moved(OVER_FIRST, NSEventModifierFlags::empty());
        let moved = self.last_pointer_input();
        let expected = PointerInput {
            kind: PointerInputKind::Moved,
            position: OVER_FIRST,
            modifiers: Modifiers::NONE,
        };
        self.expectations.expect(
            moved == Some(expected),
            "a mouse move reaches the tree as a move at the point of the surface",
            &format!("{moved:?}"),
        );
        let (first, root) = (self.states(self.first), self.states(self.root));
        self.expectations.expect(
            first.contains(States::HOVER) && root.contains(States::HOVER),
            "the button under the pointer and the root are hovered",
            &format!("{first:?} and {root:?}"),
        );
    }

    fn a_press_a_drag_and_a_release_hold_the_pointer(&mut self) {
        let none = NSEventModifierFlags::empty();
        self.source.mouse(NSEventType::LeftMouseDown, OVER_FIRST, none, 1);
        let pressed = self.last_pointer_input().map(|input| input.kind);
        let primary_press = PointerInputKind::Pressed {
            button: PointerButton::Primary,
            click_count: 1,
        };
        self.expectations.expect(
            pressed == Some(primary_press),
            "a left mouse-down is a press of the primary button",
            &format!("{pressed:?}"),
        );
        let first = self.states(self.first);
        self.expectations.expect(
            first.contains(States::ACTIVE.union(States::FOCUS)) && !first.contains(States::FOCUS_VISIBLE),
            "the pressed button is active and focused, and the pointer shows no focus",
            &format!("{first:?}"),
        );

        self.source.mouse(NSEventType::LeftMouseDragged, OVER_SECOND, none, 0);
        let (first, second) = (self.states(self.first), self.states(self.second));
        self.expectations.expect(
            first.contains(States::ACTIVE) && !first.contains(States::HOVER) && !second.contains(States::HOVER),
            "a drag keeps the pressed button active and hovers nothing outside it",
            &format!("{first:?} and {second:?}"),
        );

        self.source.mouse(NSEventType::LeftMouseUp, OVER_SECOND, none, 1);
        let (first, second) = (self.states(self.first), self.states(self.second));
        self.expectations.expect(
            !first.contains(States::ACTIVE) && second.contains(States::HOVER),
            "the release ends the press and hovers the button under the pointer",
            &format!("{first:?} and {second:?}"),
        );
        self.expectations.expect(
            !self.was_clicked(self.first, 1) && !self.was_clicked(self.second, 1),
            "a press on one button released over the other clicks neither",
            "a button heard a click",
        );
    }

    fn a_double_click_carries_its_count(&mut self) {
        let none = NSEventModifierFlags::empty();
        self.source.mouse(NSEventType::LeftMouseDown, OVER_SECOND, none, 2);
        self.source.mouse(NSEventType::LeftMouseUp, OVER_SECOND, none, 2);
        self.expectations.expect(
            self.was_clicked(self.second, 2),
            "the click count of a double click reaches the clicked button",
            "the second button heard no click of two presses",
        );
        let focused = self.focused();
        self.expectations.expect(
            focused == Some(self.second),
            "the press moved focus to the second button",
            &format!("{focused:?}"),
        );
    }

    fn the_other_buttons_are_told_apart(&mut self) {
        let none = NSEventModifierFlags::empty();
        let control_shift = NSEventModifierFlags::Control | NSEventModifierFlags::Shift;
        self.source
            .mouse(NSEventType::RightMouseDown, OVER_FIRST, control_shift, 1);
        let secondary = self.last_pointer_input();
        let expected = PointerInput {
            kind: PointerInputKind::Pressed {
                button: PointerButton::Secondary,
                click_count: 1,
            },
            position: OVER_FIRST,
            modifiers: Modifiers::CONTROL.union(Modifiers::SHIFT),
        };
        self.expectations.expect(
            secondary == Some(expected),
            "a right mouse-down is a press of the secondary button with the modifiers held",
            &format!("{secondary:?}"),
        );
        self.source.mouse(NSEventType::RightMouseUp, OVER_FIRST, none, 1);

        self.source
            .mouse(NSEventType::OtherMouseDown, OVER_FIRST, NSEventModifierFlags::Option, 1);
        let auxiliary = self.last_pointer_input();
        let auxiliary_press = PointerInputKind::Pressed {
            button: PointerButton::Auxiliary,
            click_count: 1,
        };
        self.expectations.expect(
            auxiliary.is_some_and(|input| input.kind == auxiliary_press && input.modifiers == Modifiers::OPTION),
            "another mouse-down is a press of an auxiliary button",
            &format!("{auxiliary:?}"),
        );
        self.source.mouse(NSEventType::OtherMouseUp, OVER_FIRST, none, 1);
        let (first, focused) = (self.states(self.first), self.focused());
        self.expectations.expect(
            !first.contains(States::ACTIVE) && focused == Some(self.first),
            "a press of another button gives focus and makes nothing active",
            &format!("{first:?} with focus on {focused:?}"),
        );
    }

    fn an_exit_ends_hover(&mut self) {
        self.source.mouse_exited(OVER_FIRST);
        let exited = self.last_pointer_input().map(|input| input.kind);
        let (first, root) = (self.states(self.first), self.states(self.root));
        self.expectations.expect(
            exited == Some(PointerInputKind::Exited) && !first.contains(States::HOVER) && !root.contains(States::HOVER),
            "the pointer leaving the surface ends hover",
            &format!("{exited:?} with {first:?} and {root:?}"),
        );
    }

    fn tab_moves_focus_and_shows_it(&mut self) {
        self.source.key_down(0x30, "\t", NSEventModifierFlags::empty());
        let tab = self.last_key_input();
        let expected = KeyEvent {
            phase: KeyPhase::Pressed,
            key: Key::Tab,
            modifiers: Modifiers::NONE,
            is_repeat: false,
        };
        self.expectations.expect(
            tab == Some(expected),
            "a key-down of the Tab key reaches the tree as a press of Tab",
            &format!("{tab:?}"),
        );
        let (focused, second) = (self.focused(), self.states(self.second));
        self.expectations.expect(
            focused == Some(self.second) && second.contains(States::FOCUS_VISIBLE),
            "Tab moves focus on from the first button to the second and shows it",
            &format!("{focused:?} with {second:?}"),
        );

        self.source.key_down(0x30, "\u{19}", NSEventModifierFlags::Shift);
        let focused = self.focused();
        self.expectations.expect(
            focused == Some(self.first),
            "Shift-Tab moves focus back",
            &format!("{focused:?}"),
        );
    }

    fn a_chord_and_a_character_are_told(&mut self) {
        self.source.key_down(0x01, "s", NSEventModifierFlags::Command);
        let shortcuts = self.window.update(|content| content.heard.shortcuts.clone());
        self.expectations.expect(
            shortcuts == [self.save],
            "Command-S is the chord of the shortcut",
            &format!("{shortcuts:?}"),
        );

        self.source.key_down(0x00, "A", NSEventModifierFlags::Shift);
        let shifted = self.last_key_input();
        self.expectations.expect(
            shifted.is_some_and(|event| event.key == Key::Character('a') && event.modifiers == Modifiers::SHIFT),
            "a key that types a character is named by the character in lower case",
            &format!("{shifted:?}"),
        );
        let route = self.window.update(|content| {
            let offered = &content.heard.key_events;
            offered[offered.len() - 2..]
                .iter()
                .map(|(element, _)| *element)
                .collect::<Vec<_>>()
        });
        self.expectations.expect(
            route == [self.first, self.root],
            "a key is offered to the focused button and then to the root",
            &format!("{route:?}"),
        );
    }

    fn the_named_keys_are_told_by_their_codes(&mut self) {
        let none = NSEventModifierFlags::empty();
        for (key_code, characters, key) in NAMED_KEYS {
            self.source.key_down(key_code, characters, none);
            let named = self.last_key_input().map(|event| event.key);
            self.expectations.expect(
                named == Some(key),
                &format!("the key with the code {key_code:#04x} is {key:?}"),
                &format!("{named:?}"),
            );
        }

        self.source.key(NSEventType::KeyDown, 0x7B, "\u{f702}", none, true);
        let repeated = self.last_key_input();
        self.expectations.expect(
            repeated.is_some_and(|event| event.is_repeat && event.key == Key::ArrowLeft),
            "a key held down repeats",
            &format!("{repeated:?}"),
        );
        self.source.key(NSEventType::KeyUp, 0x7B, "\u{f702}", none, false);
        let released = self.last_key_input();
        self.expectations.expect(
            released.is_some_and(|event| event.phase == KeyPhase::Released && event.key == Key::ArrowLeft),
            "a key-up reaches the tree as a release",
            &format!("{released:?}"),
        );
    }
}

fn main() -> ExitCode {
    let main_thread = MainThreadMarker::new().expect("the test runs on the main thread");
    let application = Application::new().expect("this Mac runs Metal 4");
    let mut stage = Stage::open(&application, main_thread);

    stage.the_first_frame_lays_the_tree_out();
    stage.a_move_hovers_what_is_under_the_pointer();
    stage.a_press_a_drag_and_a_release_hold_the_pointer();
    stage.a_double_click_carries_its_count();
    stage.the_other_buttons_are_told_apart();
    stage.an_exit_ends_hover();
    stage.tab_moves_focus_and_shows_it();
    stage.a_chord_and_a_character_are_told();
    stage.the_named_keys_are_told_by_their_codes();

    stage.expectations.report("event translation")
}
