use morok_core::{IconId, ImageId};

use crate::render::glyph_key::GlyphKey;

/// What one entry of an atlas holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum AtlasKey {
    Glyph(GlyphKey),
    /// An icon rasterized to the given size in device pixels.
    Icon {
        icon: IconId,
        width: u32,
        height: u32,
    },
    /// The whole picture of a small image.
    Image(ImageId),
}
