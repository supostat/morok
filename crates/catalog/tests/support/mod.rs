use std::process::ExitCode;

use morok::core::{Point, Size};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, msg_send};
use objc2_app_kit::{NSApplication, NSEvent, NSEventModifierFlags, NSEventType, NSScreen, NSView, NSWindow};
use objc2_core_foundation::CGPoint;
use objc2_core_graphics::{CGEvent, CGScrollEventUnit};
use objc2_foundation::{NSArray, NSNumber, NSPoint, NSString};

/// What a test that runs on the main thread expected and did not see. Such a test checks
/// everything it can in one run of the application and reports at the end.
#[derive(Debug, Default)]
pub(crate) struct Expectations {
    failures: Vec<String>,
}

impl Expectations {
    pub(crate) fn expect(&mut self, holds: bool, expectation: &str, observed: &str) {
        if !holds {
            self.failures.push(format!("{expectation}: {observed}"));
        }
    }

    pub(crate) fn report(self, test_name: &str) -> ExitCode {
        if self.failures.is_empty() {
            println!("{test_name}: every expectation holds");
            ExitCode::SUCCESS
        } else {
            eprintln!("{test_name} failed:\n{}", self.failures.join("\n"));
            ExitCode::FAILURE
        }
    }
}

/// A key of the keyboard: its virtual key code, what it types and the modifiers held with it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct KeyStroke {
    pub(crate) key_code: u16,
    pub(crate) characters: &'static str,
    pub(crate) modifiers: NSEventModifierFlags,
}

impl KeyStroke {
    const fn plain(key_code: u16, characters: &'static str) -> Self {
        Self {
            key_code,
            characters,
            modifiers: NSEventModifierFlags::empty(),
        }
    }

    pub(crate) const RETURN: Self = Self::plain(0x24, "\r");
    pub(crate) const ESCAPE: Self = Self::plain(0x35, "\u{1b}");
    pub(crate) const DOWN: Self = Self::plain(0x7D, "\u{f701}");
    pub(crate) const END: Self = Self::plain(0x77, "\u{f72b}");

    /// The key of a lowercase letter on the ANSI layout.
    pub(crate) fn letter(letter: char) -> Self {
        const LETTERS: [(char, u16, &str); 26] = [
            ('a', 0x00, "a"),
            ('b', 0x0B, "b"),
            ('c', 0x08, "c"),
            ('d', 0x02, "d"),
            ('e', 0x0E, "e"),
            ('f', 0x03, "f"),
            ('g', 0x05, "g"),
            ('h', 0x04, "h"),
            ('i', 0x22, "i"),
            ('j', 0x26, "j"),
            ('k', 0x28, "k"),
            ('l', 0x25, "l"),
            ('m', 0x2E, "m"),
            ('n', 0x2D, "n"),
            ('o', 0x1F, "o"),
            ('p', 0x23, "p"),
            ('q', 0x0C, "q"),
            ('r', 0x0F, "r"),
            ('s', 0x01, "s"),
            ('t', 0x11, "t"),
            ('u', 0x20, "u"),
            ('v', 0x09, "v"),
            ('w', 0x0D, "w"),
            ('x', 0x07, "x"),
            ('y', 0x10, "y"),
            ('z', 0x06, "z"),
        ];
        let (_, key_code, characters) = LETTERS
            .into_iter()
            .find(|(typed, _, _)| *typed == letter)
            .unwrap_or_else(|| panic!("`{letter}` is a lowercase letter"));
        Self::plain(key_code, characters)
    }
}

/// Makes the events `AppKit` would send for a mouse, a wheel and a keyboard and gives them to a
/// window, which finds the view under the point or its first responder as it does for the
/// events of the system.
pub(crate) struct EventSource {
    window: Retained<NSWindow>,
    content_size: Size,
}

impl EventSource {
    /// The source of the events of `window`, whose surface takes `content_size` CSS pixels.
    pub(crate) fn for_window(window: Retained<NSWindow>, content_size: Size) -> Self {
        Self { window, content_size }
    }

    /// The view that shows the surface of the window.
    pub(crate) fn surface_view(&self) -> Retained<NSView> {
        self.window.contentView().expect("the window shows its surface")
    }

    /// A point of the surface in the coordinates of the window.
    fn in_window(&self, at: Point) -> NSPoint {
        NSPoint::new(f64::from(at.x), f64::from(self.content_size.height - at.y))
    }

    fn mouse_event(&self, kind: NSEventType, at: Point) -> Retained<NSEvent> {
        NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
            kind,
            self.in_window(at),
            NSEventModifierFlags::empty(),
            0.0,
            self.window.windowNumber(),
            None,
            0,
            1,
            1.0,
        )
        .expect("the mouse event is made")
    }

    /// The primary button goes down and up at a point of the surface.
    pub(crate) fn click(&self, at: Point) {
        for kind in [NSEventType::LeftMouseDown, NSEventType::LeftMouseUp] {
            self.window.sendEvent(&self.mouse_event(kind, at));
        }
    }

    /// A key goes down and up.
    pub(crate) fn press(&self, stroke: KeyStroke) {
        let characters = NSString::from_str(stroke.characters);
        for kind in [NSEventType::KeyDown, NSEventType::KeyUp] {
            let event = NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
                kind,
                NSPoint::ZERO,
                stroke.modifiers,
                0.0,
                self.window.windowNumber(),
                None,
                &characters,
                &characters,
                false,
                stroke.key_code,
            )
            .expect("the key event is made");
            self.window.sendEvent(&event);
        }
    }

    /// Types the lowercase letters a key at a time.
    pub(crate) fn type_letters(&self, letters: &str) {
        for letter in letters.chars() {
            self.press(KeyStroke::letter(letter));
        }
    }

    /// The wheel of the mouse turns over a point of the surface: the content under it moves up
    /// by `pixels`, as a wheel that scrolls by pixels moves it. The event goes to the view of
    /// the surface as the window hands it on. An event Core Graphics makes belongs to no
    /// window, and `AppKit` reads its location as a point of the window it is given to: the
    /// location is set to the point of the window, flipped as the screen is.
    pub(crate) fn scroll_down(&self, main_thread: MainThreadMarker, at: Point, pixels: i32) {
        let event = CGEvent::new_scroll_wheel_event2(None, CGScrollEventUnit::Pixel, 1, -pixels, 0, 0)
            .expect("the wheel event is made");
        let in_window = self.in_window(at);
        let primary_screen_height = NSScreen::screens(main_thread)
            .firstObject()
            .expect("a Mac has a screen")
            .frame()
            .size
            .height;
        CGEvent::set_location(
            Some(&event),
            CGPoint::new(in_window.x, primary_screen_height - in_window.y),
        );
        let event = NSEvent::eventWithCGEvent(&event).expect("a wheel event of Core Graphics is an event of AppKit");
        self.surface_view().scrollWheel(&event);
    }
}

/// The windows of the application that are on screen.
pub(crate) fn visible_windows(main_thread: MainThreadMarker) -> Vec<Retained<NSWindow>> {
    NSApplication::sharedApplication(main_thread)
        .windows()
        .to_vec()
        .into_iter()
        .filter(|window| window.isVisible())
        .collect()
}

/// The window on screen that carries the title.
pub(crate) fn window_titled(main_thread: MainThreadMarker, title: &str) -> Option<Retained<NSWindow>> {
    visible_windows(main_thread)
        .into_iter()
        .find(|window| window.title().to_string() == title)
}

/// An accessibility element of the system, asked as an assistive technology asks it.
pub(crate) struct AccessibleElement(Retained<AnyObject>);

impl AccessibleElement {
    /// The element of a view.
    pub(crate) fn of_view(view: Retained<NSView>) -> Self {
        Self(Retained::into_super(Retained::into_super(Retained::into_super(view))))
    }

    pub(crate) fn role(&self) -> Option<String> {
        // SAFETY: an accessibility element answers `accessibilityRole` with a string or nil.
        text_of(unsafe { msg_send![&*self.0, accessibilityRole] })
    }

    pub(crate) fn title(&self) -> Option<String> {
        // SAFETY: an accessibility element answers `accessibilityTitle` with a string or nil.
        text_of(unsafe { msg_send![&*self.0, accessibilityTitle] })
    }

    /// The value of the element: its text, or its number written out.
    pub(crate) fn value(&self) -> Option<String> {
        // SAFETY: an accessibility element answers `accessibilityValue` with an object or nil.
        let value: Option<Retained<AnyObject>> = unsafe { msg_send![&*self.0, accessibilityValue] };
        let value = value?;
        match value.downcast::<NSString>() {
            Ok(text) => Some(text.to_string()),
            Err(value) => value
                .downcast::<NSNumber>()
                .ok()
                .map(|number| number.stringValue().to_string()),
        }
    }

    pub(crate) fn children(&self) -> Vec<Self> {
        // SAFETY: an accessibility element answers `accessibilityChildren` with an array or nil.
        let children: Option<Retained<NSArray<AnyObject>>> = unsafe { msg_send![&*self.0, accessibilityChildren] };
        children.map_or_else(Vec::new, |children| children.to_vec().into_iter().map(Self).collect())
    }

    /// The element and every element under it, the element first.
    pub(crate) fn descendants(self) -> Vec<Self> {
        let mut found = Vec::new();
        let mut unvisited = vec![self];
        while let Some(element) = unvisited.pop() {
            unvisited.extend(element.children());
            found.push(element);
        }
        found
    }
}

/// The text an attribute holds; none for an attribute that is not set or holds no text.
fn text_of(attribute: Option<Retained<AnyObject>>) -> Option<String> {
    attribute
        .and_then(|value| value.downcast::<NSString>().ok())
        .map(|text| text.to_string())
}
