use morok_core::{Clipboard, Payload, PayloadType};
use objc2::rc::Retained;
use objc2_app_kit::NSPasteboard;

use crate::pasteboard_payload;

/// The clipboard of the system: a pasteboard, read and written in the payload types of the
/// kit — plain text, files, addresses and data of the application's own types.
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
    fn read(&mut self, types: &[PayloadType]) -> Payload {
        pasteboard_payload::read(&self.pasteboard, types)
    }

    fn write(&mut self, payload: &Payload) {
        pasteboard_payload::write(&self.pasteboard, payload);
    }
}
