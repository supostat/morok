use std::fmt::Debug;

use crate::transfer::payload::Payload;
use crate::transfer::payload_type::PayloadType;

/// The port through which a tree reaches the clipboard of the system. The tree knows nothing
/// of pasteboards: the platform implements the port.
pub trait Clipboard: Debug {
    /// What the clipboard holds in the given types. A type it holds nothing in is left out of
    /// the payload.
    fn read(&mut self, types: &[PayloadType]) -> Payload;

    /// Puts the payload on the clipboard, in every type it holds, in place of what the
    /// clipboard held.
    fn write(&mut self, payload: &Payload);
}
