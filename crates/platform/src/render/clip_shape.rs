use bytemuck::{Pod, Zeroable};

use crate::render::rounded_rect::RoundedRect;

/// A clip as the shaders read it: a rounded rectangle in the coordinates its transform maps to
/// the surface.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct ClipShape {
    pub(crate) shape: RoundedRect,
    /// In the transform buffer of the frame.
    pub(crate) transform: u32,
    pub(crate) padding: [u32; 3],
}
