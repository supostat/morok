use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_core_text::CTFontSymbolicTraits;
use objc2_metal::{MTLDevice, MTLPixelFormat};

use crate::atlas_statistics::AtlasStatistics;
use crate::render::atlas::{Atlas, AtlasEntry};
use crate::render::atlas_kind::AtlasKind;
use crate::render::glyph_hasher::GlyphSet;
use crate::render::glyph_key::GlyphKey;
use crate::render::glyph_rasterizer::rasterize;
use crate::renderer_options::RendererOptions;
use crate::text::font_library::font_library;

const MASK_FORMAT: MTLPixelFormat = MTLPixelFormat::R8Unorm;
const MASK_BYTES_PER_TEXEL: usize = 1;
const COLOR_FORMAT: MTLPixelFormat = MTLPixelFormat::BGRA8Unorm;
const COLOR_BYTES_PER_TEXEL: usize = 4;

/// A glyph in the atlas that holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PlacedGlyph {
    pub(crate) atlas: AtlasKind,
    pub(crate) entry: AtlasEntry,
}

/// The glyphs every surface of a renderer draws: rasterized when first drawn, kept in the
/// mask atlas or, for a color font, in the color atlas.
#[derive(Debug)]
pub(crate) struct GlyphAtlases {
    mask: Atlas,
    color: Atlas,
    /// The glyphs known to have no pixels.
    blank_glyphs: GlyphSet,
    /// Scratch space for the pixels of the glyph being rasterized.
    pixels: Vec<u8>,
}

impl GlyphAtlases {
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
    pub(crate) fn glyph(&mut self, glyph: GlyphKey) -> Option<PlacedGlyph> {
        if let Some(entry) = self.mask.get(&glyph) {
            return Some(PlacedGlyph {
                atlas: AtlasKind::Mask,
                entry,
            });
        }
        if let Some(entry) = self.color.get(&glyph) {
            return Some(PlacedGlyph {
                atlas: AtlasKind::Color,
                entry,
            });
        }
        if self.blank_glyphs.contains(&glyph) {
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
            self.blank_glyphs.insert(glyph);
            return None;
        };
        let (kind, atlas) = if in_color {
            (AtlasKind::Color, &mut self.color)
        } else {
            (AtlasKind::Mask, &mut self.mask)
        };
        atlas
            .insert(glyph, extent, &self.pixels)
            .map(|entry| PlacedGlyph { atlas: kind, entry })
    }
}
