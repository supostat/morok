use bytemuck::{Pod, Zeroable};

/// One instanced quad of the glyph pipeline: the pixels of a glyph, texel for device pixel.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct GlyphInstance {
    /// The quad in CSS pixels on the surface, origin and size; it starts on a device pixel.
    pub(crate) bounds: [f32; 4],
    /// The color of the text, premultiplied.
    pub(crate) color: [f32; 4],
    /// The texel of the atlas page under the top left corner of the quad.
    pub(crate) atlas_origin: [f32; 2],
    /// The rounded clips the instance is masked by, in the clip buffer of the frame.
    pub(crate) clips_start: u32,
    pub(crate) clips_count: u32,
    /// Whether the texels are colors rather than coverage.
    pub(crate) is_color: u32,
    pub(crate) padding: [u32; 3],
}
