use std::ffi::c_void;
use std::ptr::NonNull;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLDevice, MTLOrigin, MTLPixelFormat, MTLRegion, MTLSize, MTLStorageMode, MTLTexture, MTLTextureDescriptor,
    MTLTextureUsage,
};

use crate::atlas_usage::AtlasUsage;
use crate::render::atlas_key::AtlasKey;
use crate::render::glyph_extent::GlyphExtent;
use crate::render::glyph_hasher::GlyphMap;
use crate::render::shared_texture::SharedTexture;
use crate::render::shelf_packer::ShelfPacker;

/// Where a glyph lies in an atlas and around its pen position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AtlasEntry {
    pub(crate) page: usize,
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) extent: GlyphExtent,
}

#[derive(Debug)]
struct AtlasPage {
    texture: SharedTexture,
    packer: ShelfPacker,
    /// The last frame that drew a glyph of the page.
    last_used: u64,
    entries: Vec<AtlasKey>,
}

/// Rasterized glyphs of one pixel format in shelf-packed pages. Pages are added up to a
/// budget; past it the page no frame has used for the longest is dropped whole, and its
/// glyphs are rasterized again when they are next drawn.
#[derive(Debug)]
pub(crate) struct Atlas {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    pixel_format: MTLPixelFormat,
    bytes_per_texel: usize,
    page_side: u32,
    page_budget: usize,
    pages: Vec<AtlasPage>,
    entries: GlyphMap<AtlasEntry>,
    frame: u64,
    rasterized_glyphs: u64,
    evicted_pages: u64,
}

impl Atlas {
    pub(crate) fn new(
        device: &Retained<ProtocolObject<dyn MTLDevice>>,
        pixel_format: MTLPixelFormat,
        bytes_per_texel: usize,
        page_side: u32,
        page_budget: usize,
    ) -> Self {
        Self {
            device: device.clone(),
            pixel_format,
            bytes_per_texel,
            page_side,
            page_budget,
            pages: Vec::new(),
            entries: GlyphMap::default(),
            frame: 0,
            rasterized_glyphs: 0,
            evicted_pages: 0,
        }
    }

    /// From here on the pages a glyph is taken from are in use: they are not dropped before
    /// the next frame begins.
    pub(crate) fn begin_frame(&mut self) {
        self.frame += 1;
    }

    pub(crate) fn get(&mut self, key: &AtlasKey) -> Option<AtlasEntry> {
        let entry = *self.entries.get(key)?;
        self.pages[entry.page].last_used = self.frame;
        Some(entry)
    }

    /// Stores the pixels of an entry, rows from the top. Returns none for one larger than a
    /// page: it has no place in an atlas.
    pub(crate) fn insert(&mut self, key: AtlasKey, extent: GlyphExtent, pixels: &[u8]) -> Option<AtlasEntry> {
        if extent.width > self.page_side || extent.height > self.page_side {
            return None;
        }
        let (page, x, y) = self.place(extent.width, extent.height);
        let held = &mut self.pages[page];
        held.last_used = self.frame;
        held.entries.push(key);
        let region = MTLRegion {
            origin: MTLOrigin {
                x: x as usize,
                y: y as usize,
                z: 0,
            },
            size: MTLSize {
                width: extent.width as usize,
                height: extent.height as usize,
                depth: 1,
            },
        };
        // SAFETY: the pixels are `height` rows of `width` texels of this format, and the
        // region lies inside the page.
        unsafe {
            held.texture.0.replaceRegion_mipmapLevel_withBytes_bytesPerRow(
                region,
                0,
                NonNull::new(pixels.as_ptr().cast_mut().cast::<c_void>()).expect("a slice is not null"),
                extent.width as usize * self.bytes_per_texel,
            );
        }
        let entry = AtlasEntry { page, x, y, extent };
        self.entries.insert(key, entry);
        self.rasterized_glyphs += 1;
        Some(entry)
    }

    pub(crate) fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub(crate) fn texture(&self, page: usize) -> &ProtocolObject<dyn MTLTexture> {
        &self.pages[page].texture.0
    }

    pub(crate) fn usage(&self) -> AtlasUsage {
        AtlasUsage {
            pages: self.pages.len(),
            shelves: self.pages.iter().map(|page| page.packer.shelf_count()).sum(),
            glyphs: self.entries.len(),
            rasterized_glyphs: self.rasterized_glyphs,
            evicted_pages: self.evicted_pages,
        }
    }

    /// A place of the given size: on a page that has room, on a new page while the budget
    /// lasts, else on the least recently used page, emptied. A page the frame in progress
    /// draws from is never emptied: the frame then gets a page over the budget.
    fn place(&mut self, width: u32, height: u32) -> (usize, u32, u32) {
        for (position, page) in self.pages.iter_mut().enumerate() {
            if let Some((x, y)) = page.packer.place(width, height) {
                return (position, x, y);
            }
        }
        let least_recently_used = self
            .pages
            .iter()
            .enumerate()
            .filter(|(_, page)| page.last_used < self.frame)
            .min_by_key(|(_, page)| page.last_used)
            .map(|(position, _)| position);
        let page = match least_recently_used {
            Some(position) if self.pages.len() >= self.page_budget => {
                self.evict(position);
                position
            }
            _ => self.add_page(),
        };
        let (x, y) = self.pages[page]
            .packer
            .place(width, height)
            .expect("an empty page holds an entry no larger than itself");
        (page, x, y)
    }

    fn evict(&mut self, page: usize) {
        let held = &mut self.pages[page];
        for key in held.entries.drain(..) {
            self.entries.remove(&key);
        }
        held.packer.clear();
        self.evicted_pages += 1;
    }

    fn add_page(&mut self) -> usize {
        let side = self.page_side as usize;
        // SAFETY: the format is a color format and the size is not zero.
        let descriptor = unsafe {
            MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
                self.pixel_format,
                side,
                side,
                false,
            )
        };
        descriptor.setStorageMode(MTLStorageMode::Shared);
        descriptor.setUsage(MTLTextureUsage::ShaderRead);
        let texture = self
            .device
            .newTextureWithDescriptor(&descriptor)
            .expect("Metal creates an atlas page");
        self.pages.push(AtlasPage {
            texture: SharedTexture(texture),
            packer: ShelfPacker::new(self.page_side),
            last_used: self.frame,
            entries: Vec::new(),
        });
        self.pages.len() - 1
    }
}
