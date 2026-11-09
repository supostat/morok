use morok_core::{IconId, ImageId};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_core_text::CTFontSymbolicTraits;
use objc2_metal::{MTLDevice, MTLPixelFormat};

use crate::atlas_statistics::AtlasStatistics;
use crate::media::decoded_image::DecodedImage;
use crate::media::icon_library::IconLibrary;
use crate::render::atlas::{Atlas, AtlasEntry};
use crate::render::atlas_key::AtlasKey;
use crate::render::atlas_kind::AtlasKind;
use crate::render::glyph_extent::GlyphExtent;
use crate::render::glyph_hasher::GlyphSet;
use crate::render::glyph_key::GlyphKey;
use crate::render::glyph_rasterizer::rasterize;
use crate::renderer_options::RendererOptions;
use crate::text::font_library::font_library;

const MASK_FORMAT: MTLPixelFormat = MTLPixelFormat::R8Unorm;
const MASK_BYTES_PER_TEXEL: usize = 1;
const COLOR_FORMAT: MTLPixelFormat = MTLPixelFormat::BGRA8Unorm;
const COLOR_BYTES_PER_TEXEL: usize = 4;

/// An entry in the atlas that holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PlacedEntry {
    pub(crate) atlas: AtlasKind,
    pub(crate) entry: AtlasEntry,
}

/// The glyphs, icons and small images every surface of a renderer draws: rasterized or
/// uploaded when first drawn, kept in the mask atlas when they are coverage to tint, in the
/// color atlas when they have colors of their own.
#[derive(Debug)]
pub(crate) struct Atlases {
    mask: Atlas,
    color: Atlas,
    /// The glyphs known to have no pixels.
    blank_glyphs: GlyphSet,
    /// Scratch space for the pixels of the glyph being rasterized.
    pixels: Vec<u8>,
    /// An image with a side past this goes into a texture of its own.
    maximum_image_side: u32,
}

/// The share of a page side an image in the atlas may take.
const ATLAS_IMAGE_SIDE_SHARE: u32 = 4;

impl Atlases {
    pub(crate) fn new(device: &Retained<ProtocolObject<dyn MTLDevice>>, options: &RendererOptions) -> Self {
        let atlas = |pixel_format, bytes_per_texel| {
            Atlas::new(
                device,
                pixel_format,
                bytes_per_texel,
                options.atlas_page_side,
                options.atlas_page_budget,
            )
        };
        Self {
            mask: atlas(MASK_FORMAT, MASK_BYTES_PER_TEXEL),
            color: atlas(COLOR_FORMAT, COLOR_BYTES_PER_TEXEL),
            blank_glyphs: GlyphSet::default(),
            pixels: Vec::new(),
            maximum_image_side: options.atlas_page_side / ATLAS_IMAGE_SIDE_SHARE,
        }
    }

    pub(crate) fn begin_frame(&mut self) {
        self.mask.begin_frame();
        self.color.begin_frame();
    }

    pub(crate) fn atlas(&self, kind: AtlasKind) -> &Atlas {
        match kind {
            AtlasKind::Mask => &self.mask,
            AtlasKind::Color => &self.color,
        }
    }

    pub(crate) fn statistics(&self) -> AtlasStatistics {
        AtlasStatistics {
            mask: self.mask.usage(),
            color: self.color.usage(),
        }
    }

    /// Where the glyph lies in an atlas, rasterizing it when no atlas holds it. Returns none
    /// for a glyph that draws nothing.
    pub(crate) fn glyph(&mut self, glyph: GlyphKey) -> Option<PlacedEntry> {
        let key = AtlasKey::Glyph(glyph);
        if let Some(placed) = self.placed(&key) {
            return Some(placed);
        }
        if self.blank_glyphs.contains(&key) {
            return None;
        }
        let library = font_library();
        let font = library.font(glyph.font);
        // SAFETY: the font is valid.
        let in_color = unsafe { font.symbolic_traits() }.contains(CTFontSymbolicTraits::TraitColorGlyphs);
        let Some(extent) = rasterize(
            font,
            f64::from(f32::from_bits(glyph.scale)),
            glyph.glyph,
            glyph.subpixel_phase,
            in_color,
            &mut self.pixels,
        ) else {
            self.blank_glyphs.insert(key);
            return None;
        };
        let (kind, atlas) = if in_color {
            (AtlasKind::Color, &mut self.color)
        } else {
            (AtlasKind::Mask, &mut self.mask)
        };
        atlas
            .insert(key, extent, &self.pixels)
            .map(|entry| PlacedEntry { atlas: kind, entry })
    }

    /// Where the icon lies at `width` by `height` device pixels, rasterized when no atlas
    /// holds it at that size. Returns none for an icon without pixels or larger than a page.
    pub(crate) fn icon(&mut self, icon: IconId, width: u32, height: u32, icons: &IconLibrary) -> Option<PlacedEntry> {
        let key = AtlasKey::Icon { icon, width, height };
        if let Some(placed) = self.placed(&key) {
            return Some(placed);
        }
        let raster = icons.rasterize(icon, width, height)?;
        let extent = GlyphExtent {
            width,
            height,
            left: 0,
            top: 0,
        };
        let (kind, atlas) = if raster.is_single_color {
            (AtlasKind::Mask, &mut self.mask)
        } else {
            (AtlasKind::Color, &mut self.color)
        };
        atlas
            .insert(key, extent, &raster.pixels)
            .map(|entry| PlacedEntry { atlas: kind, entry })
    }

    /// Where the picture of a small image lies in the color atlas, uploaded when the atlas does
    /// not hold it. Returns none for an image that is too large for the atlas: it gets a
    /// texture of its own.
    pub(crate) fn image(&mut self, image: ImageId, decoded: &DecodedImage) -> Option<PlacedEntry> {
        if decoded.width > self.maximum_image_side || decoded.height > self.maximum_image_side {
            return None;
        }
        let key = AtlasKey::Image(image);
        if let Some(placed) = self.placed(&key) {
            return Some(placed);
        }
        let extent = GlyphExtent {
            width: decoded.width,
            height: decoded.height,
            left: 0,
            top: 0,
        };
        self.color
            .insert(key, extent, &decoded.pixels)
            .map(|entry| PlacedEntry {
                atlas: AtlasKind::Color,
                entry,
            })
    }

    fn placed(&mut self, key: &AtlasKey) -> Option<PlacedEntry> {
        if let Some(entry) = self.mask.get(key) {
            return Some(PlacedEntry {
                atlas: AtlasKind::Mask,
                entry,
            });
        }
        self.color.get(key).map(|entry| PlacedEntry {
            atlas: AtlasKind::Color,
            entry,
        })
    }
}
