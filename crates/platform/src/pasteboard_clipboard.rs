use morok_core::Clipboard;
use objc2::rc::Retained;
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
use objc2_foundation::NSString;

/// The clipboard of the system as edited texts reach it: the plain text of a pasteboard.
#[derive(Debug)]
pub struct PasteboardClipboard {
    pasteboard: Retained<NSPasteboard>,
}

impl PasteboardClipboard {
    /// The pasteboard every application copies to and pastes from.
    pub fn general() -> Self {
        Self::of(NSPasteboard::generalPasteboard())
    }

    /// A pasteboard of its own, such as one with a unique name that no other application
    /// reads.
    pub fn of(pasteboard: Retained<NSPasteboard>) -> Self {
        Self { pasteboard }
    }
}

impl Clipboard for PasteboardClipboard {
    fn text(&mut self) -> Option<String> {
        // SAFETY: the type is a constant of AppKit.
        let string_type = unsafe { NSPasteboardTypeString };
        self.pasteboard.stringForType(string_type).map(|text| text.to_string())
    }

    fn set_text(&mut self, text: &str) {
        // SAFETY: the type is a constant of AppKit.
        let string_type = unsafe { NSPasteboardTypeString };
        self.pasteboard.clearContents();
        self.pasteboard
            .setString_forType(&NSString::from_str(text), string_type);
    }
}
