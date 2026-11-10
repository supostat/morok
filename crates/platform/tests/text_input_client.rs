mod support;

use std::ops::Range;
use std::process::ExitCode;
use std::ptr;
use std::time::Duration;

use morok_components::TextField;
use morok_components::styles::STYLE_SHEET;
use morok_components::tokens::{DARK, LIGHT};
use morok_components::view::{SurfaceId, Surfaces};
use morok_core::accesskit::{ActionRequest, TreeUpdate};
use morok_core::{
    DisplayItem, ElementId, ElementTree, FrameScheduler, KeyEvent, Point, PointerInput, Rect, ScrollInput, Size,
    TextCommand,
};
use morok_platform::{
    Application, CoreTextShaper, FocusedText, PasteboardClipboard, SurfaceContent, SurfaceSettings, Window,
    WindowOptions,
};
use morok_reactive::{Runtime, Signal};
use morok_style::ColorScheme;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, msg_send};
use objc2_app_kit::{
    NSApplication, NSEvent, NSEventModifierFlags, NSEventType, NSPasteboard, NSPasteboardTypeString, NSView, NSWindow,
};
use objc2_foundation::{NSNotFound, NSPoint, NSRange, NSRect, NSString};
use support::{Expectations, run_main_loop};

const CONTENT_SIZE: Size = Size {
    width: 320.0,
    height: 120.0,
};
/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// Long enough for a requested frame to be drawn.
const ONE_FRAME: Duration = Duration::from_millis(500);
const BACKSPACE_KEY: u16 = 0x33;

/// A window on a text field of the kit, mounted through the component model and focused.
struct FieldWindow {
    runtime: Runtime<Surfaces>,
    surfaces: Surfaces,
    surface: SurfaceId,
    text_shaper: CoreTextShaper,
    /// The line the field edits.
    input: ElementId,
    value: Signal<String>,
}

impl FieldWindow {
    fn new(
        frame_scheduler: Box<dyn FrameScheduler>,
        settings: SurfaceSettings,
        pasteboard: Retained<NSPasteboard>,
    ) -> Self {
        let mut runtime: Runtime<Surfaces> = Runtime::new();
        let mut surfaces = Surfaces::new();
        let theme = match settings.color_scheme {
            ColorScheme::Light => LIGHT,
            ColorScheme::Dark => DARK,
        };
        let surface = surfaces.open(&mut runtime, STYLE_SHEET, theme);
        let value = runtime.signal(String::from("Folder"));
        let tree = surfaces.surface_mut(surface).tree_mut();
        tree.set_frame_scheduler(frame_scheduler);
        tree.set_clipboard(Box::new(PasteboardClipboard::of(pasteboard)));
        let root = tree.root();
        let field = surfaces.mount(&mut runtime, surface, root, |mount| {
            TextField::new(value).placeholder("Folder name").mount(mount)
        });
        runtime.flush(&mut surfaces).expect("the effects of the field settle");
        let tree = surfaces.surface_mut(surface).tree_mut();
        let input = *tree.children(field).last().expect("the field holds its line");
        assert!(tree.focus(input), "the line of the field takes focus");
        Self {
            runtime,
            surfaces,
            surface,
            text_shaper: CoreTextShaper::new(),
            input,
            value,
        }
    }

    fn tree(&self) -> &ElementTree {
        self.surfaces.surface(self.surface).tree()
    }

    fn flush(&mut self) {
        self.runtime
            .flush(&mut self.surfaces)
            .expect("the effects of the field settle");
    }
}

impl SurfaceContent for FieldWindow {
    fn frame(&mut self, viewport: Size, scale: f32, now: Duration, show: &mut dyn FnMut(&ElementTree)) {
        self.surfaces.surface_mut(self.surface).refresh_hover(&mut self.runtime);
        self.flush();
        let tree = self.surfaces.surface_mut(self.surface).tree_mut();
        tree.resolve_styles(now);
        tree.compute_layout(viewport, scale, &mut self.text_shaper);
        tree.build_display_list();
        show(tree);
    }

    fn color_scheme_changed(&mut self, _color_scheme: ColorScheme) {}

    fn reduced_motion_changed(&mut self, _reduces_motion: bool) {}

    /// The field is changed through the input client alone: a pointer that happens to be over
    /// the window changes nothing.
    fn pointer_input(&mut self, _input: &PointerInput) {}

    fn scroll_input(&mut self, _input: &ScrollInput) {}

    fn key_input(&mut self, event: &KeyEvent) -> bool {
        let was_used = self
            .surfaces
            .surface_mut(self.surface)
            .key_input(&mut self.runtime, event);
        self.flush();
        was_used
    }

    fn accessibility_tree(&mut self) -> TreeUpdate {
        self.surfaces.surface_mut(self.surface).tree_mut().accessibility_tree()
    }

    fn accessibility_update(&mut self) -> TreeUpdate {
        self.surfaces
            .surface_mut(self.surface)
            .tree_mut()
            .accessibility_update()
    }

    fn accessibility_action(&mut self, _request: &ActionRequest) {}

    fn focused_text(&mut self) -> Option<FocusedText> {
        let tree = self.tree();
        let focused = tree.focused_text_input()?;
        Some(FocusedText {
            text: tree.text(focused).to_owned(),
            selection: tree.text_selection(focused),
            marked: tree.marked_text(focused),
        })
    }

    fn insert_text(&mut self, text: &str, replacement: Option<Range<usize>>) {
        self.surfaces
            .surface_mut(self.surface)
            .insert_text(&mut self.runtime, text, replacement);
        self.flush();
    }

    fn set_marked_text(&mut self, text: &str, selected: Range<usize>, replacement: Option<Range<usize>>) {
        self.surfaces
            .surface_mut(self.surface)
            .set_marked_text(&mut self.runtime, text, selected, replacement);
        self.flush();
    }

    fn unmark_text(&mut self) {
        self.surfaces.surface_mut(self.surface).unmark_text();
    }

    fn text_command(&mut self, command: TextCommand) -> bool {
        let was_taken = self
            .surfaces
            .surface_mut(self.surface)
            .text_command(&mut self.runtime, command);
        self.flush();
        was_taken
    }

    fn text_range_box(&mut self, range: Range<usize>) -> Option<Rect> {
        let tree = self.tree();
        Some(tree.text_range_box(tree.focused_text_input()?, range))
    }

    fn text_offset_at(&mut self, point: Point) -> Option<usize> {
        let tree = self.tree();
        Some(tree.text_offset_at(tree.focused_text_input()?, point))
    }
}

/// The window of the test, its view as the system reaches it, and what the test expects.
struct Stage {
    window: Window<FieldWindow>,
    native_window: Retained<NSWindow>,
    view: Retained<NSView>,
    pasteboard: Retained<NSPasteboard>,
    expectations: Expectations,
}

/// A range at `NSNotFound`: no replacement range, the selection or the marked text is replaced.
fn no_replacement() -> NSRange {
    NSRange::new(usize::try_from(NSNotFound).expect("NSNotFound is not negative"), 0)
}

impl Stage {
    fn open(main_thread: MainThreadMarker, application: &Application) -> Self {
        let pasteboard = NSPasteboard::pasteboardWithUniqueName();
        let options = WindowOptions {
            title: String::from("morok: text input client"),
            content_size: CONTENT_SIZE,
        };
        let field_pasteboard = pasteboard.clone();
        let window = Window::open(application, &options, |frame_scheduler, settings| {
            FieldWindow::new(frame_scheduler, settings, field_pasteboard)
        });
        let native_window = NSApplication::sharedApplication(main_thread)
            .windows()
            .firstObject()
            .expect("the application has opened its window");
        let view = native_window.contentView().expect("the window shows its surface");
        Self {
            window,
            native_window,
            view,
            pasteboard,
            expectations: Expectations::default(),
        }
    }

    fn client(&self) -> &AnyObject {
        &self.view
    }

    fn value(&self) -> String {
        self.window.update(|field| field.runtime.get(field.value))
    }

    fn marked_range(&self) -> NSRange {
        // SAFETY: the view is a text input client.
        unsafe { msg_send![self.client(), markedRange] }
    }

    fn selected_range(&self) -> NSRange {
        // SAFETY: the view is a text input client.
        unsafe { msg_send![self.client(), selectedRange] }
    }

    fn has_marked_text(&self) -> bool {
        // SAFETY: the view is a text input client.
        unsafe { msg_send![self.client(), hasMarkedText] }
    }

    fn set_marked_text(&self, text: &str, selected: NSRange) {
        let (client, text, none) = (self.client(), NSString::from_str(text), no_replacement());
        // SAFETY: the view is a text input client, and the marked text is a string.
        let (): () =
            unsafe { msg_send![client, setMarkedText: &*text, selectedRange: selected, replacementRange: none] };
    }

    fn insert_text(&self, text: &str) {
        let text = NSString::from_str(text);
        // SAFETY: the view is a text input client, and the inserted text is a string.
        let (): () = unsafe { msg_send![self.client(), insertText: &*text, replacementRange: no_replacement()] };
    }

    fn first_rect(&self, range: NSRange) -> NSRect {
        // SAFETY: the view is a text input client; no actual range is asked for.
        unsafe { msg_send![self.client(), firstRectForCharacterRange: range, actualRange: ptr::null_mut::<NSRange>()] }
    }

    fn press_backspace(&self) {
        let characters = NSString::from_str("\u{7f}");
        let event = NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
            NSEventType::KeyDown,
            NSPoint::ZERO,
            NSEventModifierFlags::empty(),
            0.0,
            self.native_window.windowNumber(),
            None,
            &characters,
            &characters,
            false,
            BACKSPACE_KEY,
        )
        .expect("the key event is made");
        self.native_window.sendEvent(&event);
    }

    fn the_field_shows_its_value_with_the_caret_at_its_end(&mut self) {
        run_main_loop(SETTLING);
        let selected = self.selected_range();
        self.expectations.expect(
            self.value() == "Folder" && selected == NSRange::new(6, 0) && !self.has_marked_text(),
            "the focused field holds its value with the caret after it and no marked text",
            &format!("{:?}, {selected:?}", self.value()),
        );
    }

    fn a_composition_is_marked_until_it_is_committed(&mut self) {
        self.set_marked_text("to", NSRange::new(2, 0));
        self.set_marked_text("toukyou", NSRange::new(7, 0));
        let (marked, selected) = (self.marked_range(), self.selected_range());
        self.expectations.expect(
            self.has_marked_text() && marked == NSRange::new(6, 7) && selected == NSRange::new(13, 0),
            "a composition stands marked in place of the one before, the caret where the input method put it",
            &format!("marked {marked:?}, selected {selected:?}"),
        );
        let (field_text, field_marked) = self.window.update(|field| {
            let tree = field.tree();
            (tree.text(field.input).to_owned(), tree.marked_text(field.input))
        });
        self.expectations.expect(
            field_text == "Foldertoukyou" && field_marked == Some(6..13),
            "the mounted field holds the composition, marked",
            &format!("{field_text:?}, {field_marked:?}"),
        );

        run_main_loop(ONE_FRAME);
        let candidates = self.first_rect(NSRange::new(6, 7));
        let window_frame = self.native_window.frame();
        let lies_in_the_window = candidates.size.width > 0.0
            && candidates.size.height > 0.0
            && candidates.origin.x >= window_frame.origin.x
            && candidates.origin.y >= window_frame.origin.y
            && candidates.origin.x + candidates.size.width <= window_frame.origin.x + window_frame.size.width
            && candidates.origin.y + candidates.size.height <= window_frame.origin.y + window_frame.size.height;
        self.expectations.expect(
            lies_in_the_window,
            "the window of candidates opens next to the composition, on the screen inside the window",
            &format!("{candidates:?} in {window_frame:?}"),
        );

        self.insert_text("\u{6771}\u{4EAC}");
        let selected = self.selected_range();
        self.expectations.expect(
            self.value() == "Folder\u{6771}\u{4EAC}" && !self.has_marked_text() && selected == NSRange::new(8, 0),
            "the committed text replaces the composition and reaches the value of the field",
            &format!("{:?}, {selected:?}", self.value()),
        );
    }

    fn a_key_that_is_a_command_edits_the_field(&mut self) {
        self.press_backspace();
        self.expectations.expect(
            self.value() == "Folder\u{6771}",
            "Backspace reaches the field through the input methods and deletes the character before the caret",
            &self.value(),
        );
    }

    fn a_paste_reaches_the_field(&mut self) {
        // SAFETY: the type is a constant of AppKit.
        let string_type = unsafe { NSPasteboardTypeString };
        self.pasteboard.clearContents();
        self.pasteboard
            .setString_forType(&NSString::from_str(" 2026"), string_type);
        // SAFETY: the view takes the paste action of the Edit menu with any sender.
        let (): () = unsafe { msg_send![&*self.view, paste: ptr::null::<AnyObject>()] };
        self.expectations.expect(
            self.value() == "Folder\u{6771} 2026",
            "a paste puts the text of the pasteboard at the caret",
            &self.value(),
        );

        run_main_loop(ONE_FRAME);
        let glyphs = self.window.update(|field| {
            let display_list = field.tree().display_list();
            display_list
                .items()
                .iter()
                .filter_map(|item| match item {
                    DisplayItem::GlyphRun { glyphs, .. } => Some(display_list.glyphs(*glyphs).len()),
                    _ => None,
                })
                .sum::<usize>()
        });
        self.expectations.expect(
            glyphs == "Folder\u{6771} 2026".chars().count(),
            "the next frame draws every character of the field",
            &glyphs.to_string(),
        );

        // SAFETY: the view takes the undo action of the Edit menu with any sender.
        let (): () = unsafe { msg_send![&*self.view, undo: ptr::null::<AnyObject>()] };
        self.expectations.expect(
            self.value() == "Folder\u{6771}",
            "undo takes the whole paste back",
            &self.value(),
        );
    }
}

fn main() -> ExitCode {
    let main_thread = MainThreadMarker::new().expect("the test runs on the main thread");
    let application = Application::new().expect("this Mac runs Metal 4");
    let mut stage = Stage::open(main_thread, &application);

    stage.the_field_shows_its_value_with_the_caret_at_its_end();
    stage.a_composition_is_marked_until_it_is_committed();
    stage.a_key_that_is_a_command_edits_the_field();
    stage.a_paste_reaches_the_field();

    // SAFETY: a pasteboard with a unique name is released once nothing reads it any more.
    let (): () = unsafe { msg_send![&*stage.pasteboard, releaseGlobally] };
    stage.expectations.report("text input client")
}
