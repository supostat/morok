#![allow(dead_code)]

use std::process::ExitCode;

use morok::core::{Point, Size};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, msg_send};
use objc2_app_kit::{NSApplication, NSEvent, NSEventModifierFlags, NSEventType, NSView, NSWindow};
use objc2_foundation::{NSArray, NSPoint, NSString};

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

/// Makes the events `AppKit` would send for a mouse and a keyboard and gives them to the only
/// window of the application, which finds the view under the point or its first responder as
/// it does for the events of the system.
pub(crate) struct EventSource {
    window: Retained<NSWindow>,
    content_size: Size,
}

impl EventSource {
    /// The source of the events of the only window of the application, whose surface takes
    /// `content_size` CSS pixels.
    pub(crate) fn for_the_window(main_thread: MainThreadMarker, content_size: Size) -> Self {
        let window = NSApplication::sharedApplication(main_thread)
            .windows()
            .firstObject()
            .expect("the application has opened its window");
        Self { window, content_size }
    }

    /// The view that shows the surface of the window.
    pub(crate) fn surface_view(&self) -> Retained<NSView> {
        self.window.contentView().expect("the window shows its surface")
    }

    fn mouse(&self, kind: NSEventType, at: Point) {
        let location = NSPoint::new(f64::from(at.x), f64::from(self.content_size.height - at.y));
        let event = NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
            kind,
            location,
            NSEventModifierFlags::empty(),
            0.0,
            self.window.windowNumber(),
            None,
            0,
            1,
            1.0,
        )
        .expect("the mouse event is made");
        self.window.sendEvent(&event);
    }

    /// The primary button goes down and up at a point of the surface.
    pub(crate) fn click(&self, at: Point) {
        self.mouse(NSEventType::LeftMouseDown, at);
        self.mouse(NSEventType::LeftMouseUp, at);
    }

    /// A key goes down and up: `key_code` is its virtual key code and `characters` what it
    /// types.
    pub(crate) fn press_key(&self, key_code: u16, characters: &str) {
        for kind in [NSEventType::KeyDown, NSEventType::KeyUp] {
            let characters = NSString::from_str(characters);
            let event = NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
                kind,
                NSPoint::ZERO,
                NSEventModifierFlags::empty(),
                0.0,
                self.window.windowNumber(),
                None,
                &characters,
                &characters,
                false,
                key_code,
            )
            .expect("the key event is made");
            self.window.sendEvent(&event);
        }
    }
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

    pub(crate) fn children(&self) -> Vec<Self> {
        // SAFETY: an accessibility element answers `accessibilityChildren` with an array or nil.
        let children: Option<Retained<NSArray<AnyObject>>> = unsafe { msg_send![&*self.0, accessibilityChildren] };
        children.map_or_else(Vec::new, |children| children.to_vec().into_iter().map(Self).collect())
    }

    /// Asks the element for its default action. Returns whether it performed it.
    pub(crate) fn press(&self) -> bool {
        // SAFETY: an accessibility element answers `accessibilityPerformPress` with a flag.
        unsafe { msg_send![&*self.0, accessibilityPerformPress] }
    }
}

/// The text an attribute holds; none for an attribute that is not set or holds no text.
fn text_of(attribute: Option<Retained<AnyObject>>) -> Option<String> {
    attribute
        .and_then(|value| value.downcast::<NSString>().ok())
        .map(|text| text.to_string())
}
