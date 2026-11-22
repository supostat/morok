#![allow(dead_code)]

use std::ffi::c_void;
use std::process::ExitCode;
use std::time::Duration;

use morok::core::{Point, Size};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, msg_send};
use objc2_app_kit::{
    NSApplication, NSEvent, NSEventMask, NSEventModifierFlags, NSEventType, NSScreen, NSView, NSWindow,
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_core_graphics::{
    CGBitmapContextCreate, CGColorSpace, CGContext, CGImage, CGImageAlphaInfo, CGWindowImageOption, CGWindowListOption,
    kCGColorSpaceSRGB,
};
use objc2_foundation::{NSArray, NSDate, NSDefaultRunLoopMode, NSPoint, NSString};

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

/// How long the system has at most to bring the application to the front.
const COMING_TO_FRONT: Duration = Duration::from_secs(3);

/// Brings the application to the front, as a launch by the user does, and returns whether it
/// came there: its windows then take the keyboard and show what the windows of the active
/// application show. The event loop of a test hands the application no events of the system,
/// so the events that tell it of its activation are handed to it here; the events of the mouse
/// and the keyboard stay in the queue.
pub(crate) fn bring_to_front(main_thread: MainThreadMarker) -> bool {
    let application = NSApplication::sharedApplication(main_thread);
    #[expect(
        deprecated,
        reason = "a process started behind the active application comes to the front only when it insists"
    )]
    application.activateIgnoringOtherApps(true);
    let deadline = NSDate::dateWithTimeIntervalSinceNow(COMING_TO_FRONT.as_secs_f64());
    // SAFETY: the name is a constant of Foundation.
    let mode = unsafe { NSDefaultRunLoopMode };
    while !application.isActive()
        && let Some(event) = application.nextEventMatchingMask_untilDate_inMode_dequeue(
            NSEventMask::AppKitDefined,
            Some(&deadline),
            mode,
            true,
        )
    {
        application.sendEvent(&event);
    }
    application.isActive()
}

/// The pixels the screen shows along a row of the desktop that starts at `start` and is
/// `length` points long: of `window` alone, or with the windows under it as well. One pixel to
/// a point, red, green, blue and alpha each; none when the system hands no picture of it.
pub(crate) fn screen_row(
    window: &NSWindow,
    start: NSPoint,
    length: f64,
    with_windows_under: bool,
    main_thread: MainThreadMarker,
) -> Option<Vec<[u8; 4]>> {
    let desktop_height = NSScreen::screens(main_thread).firstObject()?.frame().size.height;
    // The desktop is measured upwards from the bottom of the main screen, a capture downwards
    // from its top.
    let row = CGRect::new(
        CGPoint::new(start.x, desktop_height - start.y),
        CGSize::new(length, 1.0),
    );
    let windows = if with_windows_under {
        CGWindowListOption::OptionOnScreenBelowWindow | CGWindowListOption::OptionIncludingWindow
    } else {
        CGWindowListOption::OptionIncludingWindow
    };
    #[expect(deprecated, reason = "the workspace has no binding of ScreenCaptureKit")]
    let shown = objc2_core_graphics::CGWindowListCreateImage(
        row,
        windows,
        u32::try_from(window.windowNumber()).ok()?,
        CGWindowImageOption::NominalResolution,
    )?;
    let width = CGImage::width(Some(&shown));
    Some(pixels_of(&shown)?.into_iter().take(width).collect())
}

/// The pixels of an image drawn whole into a bitmap of its size, rows from the top.
fn pixels_of(image: &CGImage) -> Option<Vec<[u8; 4]>> {
    let (width, height) = (CGImage::width(Some(image)), CGImage::height(Some(image)));
    let mut pixels = vec![[0_u8; 4]; width * height];
    // SAFETY: the name is a constant of Core Graphics.
    let color_space = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB }));
    // SAFETY: the buffer holds `height` rows of `width` pixels of red, green, blue and alpha,
    // premultiplied, and outlives the context, which is dropped before it is read.
    let context = unsafe {
        CGBitmapContextCreate(
            pixels.as_mut_ptr().cast::<c_void>(),
            width,
            height,
            8,
            width * 4,
            color_space.as_deref(),
            CGImageAlphaInfo::PremultipliedLast.0,
        )
    }?;
    let whole = CGRect::new(CGPoint::ZERO, CGSize::new(width as f64, height as f64));
    CGContext::draw_image(Some(&context), whole, Some(image));
    drop(context);
    Some(pixels)
}

/// Makes the events `AppKit` would send for a mouse and a keyboard and gives them to a window
/// of the application, which finds the view under the point or its first responder as it
/// does for the events of the system.
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

    /// The source of the events of `window`, whose surface takes `content_size` CSS pixels.
    pub(crate) fn for_window(window: Retained<NSWindow>, content_size: Size) -> Self {
        Self { window, content_size }
    }

    /// The window the events go to.
    pub(crate) fn window(&self) -> &NSWindow {
        &self.window
    }

    /// The view that shows the surface of the window.
    pub(crate) fn surface_view(&self) -> Retained<NSView> {
        self.window.contentView().expect("the window shows its surface")
    }

    fn mouse_event(&self, kind: NSEventType, at: Point) -> Retained<NSEvent> {
        let location = NSPoint::new(f64::from(at.x), f64::from(self.content_size.height - at.y));
        NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
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
        .expect("the mouse event is made")
    }

    fn mouse(&self, kind: NSEventType, at: Point) {
        self.window.sendEvent(&self.mouse_event(kind, at));
    }

    /// The primary button goes down and up at a point of the surface.
    pub(crate) fn click(&self, at: Point) {
        self.press(at);
        self.release(at);
    }

    /// The primary button goes down at a point of the surface.
    pub(crate) fn press(&self, at: Point) {
        self.mouse(NSEventType::LeftMouseDown, at);
    }

    /// The pointer moves with the primary button down to a point the surface measures, which
    /// may lie outside the window.
    pub(crate) fn drag_to(&self, at: Point) {
        self.mouse(NSEventType::LeftMouseDragged, at);
    }

    /// The primary button comes up at a point the surface measures.
    pub(crate) fn release(&self, at: Point) {
        self.mouse(NSEventType::LeftMouseUp, at);
    }

    /// The primary button goes down at `from` and the pointer moves to `to` with the button
    /// down. The release is put into the queue of the application: a drag of the system that
    /// began on the way takes the events of the queue from there on, and ends with it.
    pub(crate) fn drag_and_release(&self, main_thread: MainThreadMarker, from: Point, to: Point) {
        self.window
            .sendEvent(&self.mouse_event(NSEventType::LeftMouseDown, from));
        self.window
            .sendEvent(&self.mouse_event(NSEventType::LeftMouseDragged, to));
        NSApplication::sharedApplication(main_thread)
            .postEvent_atStart(&self.mouse_event(NSEventType::LeftMouseUp, to), false);
    }

    /// The secondary button goes down and up at a point of the surface.
    pub(crate) fn secondary_click(&self, at: Point) {
        self.mouse(NSEventType::RightMouseDown, at);
        self.mouse(NSEventType::RightMouseUp, at);
    }

    /// A key goes down and up: `key_code` is its virtual key code and `characters` what it
    /// types.
    pub(crate) fn press_key(&self, key_code: u16, characters: &str) {
        for kind in [NSEventType::KeyDown, NSEventType::KeyUp] {
            self.window.sendEvent(&self.key_event(kind, key_code, characters));
        }
    }

    /// Puts a key that goes down and up into the queue of the application, for whatever
    /// takes the events of the queue next: a menu of the system does while it is open.
    pub(crate) fn queue_key_press(&self, main_thread: MainThreadMarker, key_code: u16, characters: &str) {
        let application = NSApplication::sharedApplication(main_thread);
        for kind in [NSEventType::KeyDown, NSEventType::KeyUp] {
            application.postEvent_atStart(&self.key_event(kind, key_code, characters), false);
        }
    }

    fn key_event(&self, kind: NSEventType, key_code: u16, characters: &str) -> Retained<NSEvent> {
        let characters = NSString::from_str(characters);
        NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
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
        .expect("the key event is made")
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
