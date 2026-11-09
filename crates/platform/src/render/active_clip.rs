use crate::render::pixel_rect::PixelRect;

/// The clip in force while a run of display items is encoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActiveClip {
    /// The pixels every enclosing clip lets through.
    pub(crate) scissor: PixelRect,
    /// The enclosing rounded clips, a run in the clip buffer of the frame.
    pub(crate) rounded_start: usize,
    pub(crate) rounded_count: usize,
}
