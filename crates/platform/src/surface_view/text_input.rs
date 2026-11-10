use std::ops::Range;

use morok_core::{KeyPhase, Point, TextCommand};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{AnyThread, DefinedClass, Message};
use objc2_app_kit::NSEvent;
use objc2_foundation::{
    NSArray, NSAttributedString, NSAttributedStringKey, NSPoint, NSRange, NSRect, NSSize, NSString,
};

use super::SurfaceView;
use super::key_routing::KeyRouting;
use crate::text_ranges::{byte_range, no_range, utf16_offset, utf16_range};

/// The view as the client of the input methods of the system: what is typed, dictated or
/// composed reaches the text the focused element edits through it, in UTF-16 ranges that the
/// view turns into the bytes of the text.
impl SurfaceView {
    /// Hands a key to the input methods while the content edits text: they turn it into text,
    /// into a composition, or into a command the content is offered as a key. Returns whether
    /// anything used the key.
    pub(super) fn interpret_key(&self, event: &NSEvent) -> bool {
        let state = self.ivars();
        state.key_in_flight.replace(Some(event.retain()));
        state.key_routing.set(KeyRouting::Waiting);
        let input_methods_took_it = self.inputContext().is_some_and(|context| context.handleEvent(event));
        state.key_in_flight.replace(None);
        match state.key_routing.replace(KeyRouting::Waiting) {
            KeyRouting::Used => true,
            KeyRouting::Unused => false,
            KeyRouting::Waiting => input_methods_took_it,
        }
    }

    /// A key the input methods made a command of goes to the content as the key it is: the
    /// content has key bindings of its own.
    pub(super) fn input_command(&self) {
        let state = self.ivars();
        let Some(event) = state.key_in_flight.borrow().clone() else {
            return;
        };
        let routing = if self.key_input(&event, KeyPhase::Pressed) {
            KeyRouting::Used
        } else {
            KeyRouting::Unused
        };
        state.key_routing.set(routing);
    }

    pub(super) fn input_insert(&self, string: &AnyObject, replacement: NSRange) {
        let inserted = plain_string(string);
        {
            let mut content = self.content().borrow_mut();
            let Some(focused) = content.focused_text() else {
                return;
            };
            content.insert_text(&inserted, byte_range(&focused.text, replacement));
        }
        self.ivars().key_routing.set(KeyRouting::Used);
        self.text_moved();
    }

    pub(super) fn input_set_marked(&self, string: &AnyObject, selected: NSRange, replacement: NSRange) {
        let marked = plain_string(string);
        {
            let mut content = self.content().borrow_mut();
            let Some(focused) = content.focused_text() else {
                return;
            };
            let selected = byte_range(&marked, selected).unwrap_or(marked.len()..marked.len());
            content.set_marked_text(&marked, selected, byte_range(&focused.text, replacement));
        }
        self.ivars().key_routing.set(KeyRouting::Used);
        self.text_moved();
    }

    pub(super) fn input_unmark(&self) {
        self.content().borrow_mut().unmark_text();
        self.text_moved();
    }

    pub(super) fn input_selected_range(&self) -> NSRange {
        self.content()
            .borrow_mut()
            .focused_text()
            .map_or_else(no_range, |focused| {
                utf16_range(&focused.text, &focused.selection.range())
            })
    }

    pub(super) fn input_marked_range(&self) -> NSRange {
        self.content()
            .borrow_mut()
            .focused_text()
            .and_then(|focused| Some(utf16_range(&focused.text, &focused.marked?)))
            .unwrap_or_else(no_range)
    }

    pub(super) fn input_has_marked_text(&self) -> bool {
        self.content()
            .borrow_mut()
            .focused_text()
            .is_some_and(|focused| focused.marked.is_some())
    }

    /// The part of the focused text a range names, kept inside the text; `actual_range`, when
    /// given, learns the range that part takes.
    pub(super) fn input_substring(
        &self,
        range: NSRange,
        actual_range: *mut NSRange,
    ) -> Option<Retained<NSAttributedString>> {
        let focused = self.content().borrow_mut().focused_text()?;
        let bytes = byte_range(&focused.text, range)?;
        write_actual_range(actual_range, utf16_range(&focused.text, &bytes));
        let substring = NSString::from_str(&focused.text[bytes]);
        Some(NSAttributedString::initWithString(
            NSAttributedString::alloc(),
            &substring,
        ))
    }

    pub(super) fn input_valid_attributes() -> Retained<NSArray<NSAttributedStringKey>> {
        NSArray::new()
    }

    /// Where the first line of a range of the focused text lies on the screen: the window of
    /// candidates of an input method opens next to it.
    pub(super) fn input_first_rect(&self, range: NSRange, actual_range: *mut NSRange) -> NSRect {
        let mut content = self.content().borrow_mut();
        let Some(focused) = content.focused_text() else {
            return NSRect::ZERO;
        };
        let bytes: Range<usize> = byte_range(&focused.text, range).unwrap_or_else(|| focused.selection.range());
        write_actual_range(actual_range, utf16_range(&focused.text, &bytes));
        let Some(text_box) = content.text_range_box(bytes) else {
            return NSRect::ZERO;
        };
        drop(content);
        let in_view = NSRect::new(
            NSPoint::new(f64::from(text_box.x), f64::from(text_box.y)),
            NSSize::new(f64::from(text_box.width), f64::from(text_box.height)),
        );
        let in_window = self.convertRect_toView(in_view, None);
        self.window()
            .map_or(in_window, |window| window.convertRectToScreen(in_window))
    }

    /// The place of the focused text nearest to a point of the screen, in UTF-16 code units.
    pub(super) fn input_character_index(&self, screen_point: NSPoint) -> usize {
        let Some(window) = self.window() else {
            return no_range().location;
        };
        let in_view = self.convertPoint_fromView(window.convertPointFromScreen(screen_point), None);
        let mut content = self.content().borrow_mut();
        let point = Point {
            x: in_view.x as f32,
            y: in_view.y as f32,
        };
        match (content.focused_text(), content.text_offset_at(point)) {
            (Some(focused), Some(offset)) => utf16_offset(&focused.text, offset),
            _ => no_range().location,
        }
    }

    /// A command of the Edit menu, which reaches the view along the responder chain.
    pub(super) fn input_text_command(&self, command: TextCommand) {
        self.content().borrow_mut().text_command(command);
        self.text_moved();
    }

    /// The text or its caret moved: the window of candidates follows it.
    fn text_moved(&self) {
        if let Some(context) = self.inputContext() {
            context.invalidateCharacterCoordinates();
        }
    }
}

/// The characters of a plain or an attributed string, as an input method hands them over.
fn plain_string(string: &AnyObject) -> String {
    if let Some(attributed) = string.downcast_ref::<NSAttributedString>() {
        return attributed.string().to_string();
    }
    string
        .downcast_ref::<NSString>()
        .map_or_else(String::new, ToString::to_string)
}

fn write_actual_range(actual_range: *mut NSRange, range: NSRange) {
    if !actual_range.is_null() {
        // SAFETY: AppKit passes a pointer to a range it owns, or null.
        unsafe { actual_range.write(range) };
    }
}
