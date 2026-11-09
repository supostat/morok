use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ptr;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};

use morok_core::{FontId, FontRequest};
use morok_style::{FontStyle, FontVariantNumeric};
use objc2_core_foundation::Type;
use objc2_core_text::CTFont;

use crate::text::font_resolution;
use crate::text::shared_font::SharedFont;

static FONT_LIBRARY: LazyLock<Mutex<FontLibrary>> = LazyLock::new(Mutex::default);

/// The fonts of the process. A shaper names the fonts it sets text in by their place here,
/// and the renderer finds the font of a glyph run by that name.
pub(crate) fn font_library() -> MutexGuard<'static, FontLibrary> {
    FONT_LIBRARY.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What tells two font requests apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct RequestKey {
    families: u64,
    size: u32,
    weight: u16,
    style: FontStyle,
    numeric: FontVariantNumeric,
}

impl RequestKey {
    fn of(request: &FontRequest<'_>) -> Self {
        let mut families = DefaultHasher::new();
        request.families.hash(&mut families);
        Self {
            families: families.finish(),
            size: request.size.to_bits(),
            weight: request.weight.0,
            style: request.style,
            numeric: request.numeric,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct FontLibrary {
    fonts: Vec<SharedFont>,
    by_request: HashMap<RequestKey, FontId>,
    /// The fonts Core Text handed out, by the address of the object. Core Text may hand out
    /// several objects for one font, so each object met is held here to keep its address its own.
    by_address: HashMap<usize, FontId>,
    held_objects: Vec<SharedFont>,
}

impl FontLibrary {
    /// The font a style asks for: the first of its families the system has, in its weight,
    /// style and numeric variant.
    pub(crate) fn resolve(&mut self, request: &FontRequest<'_>) -> FontId {
        let key = RequestKey::of(request);
        if let Some(font) = self.by_request.get(&key) {
            return *font;
        }
        let font = self.name(&font_resolution::resolve(request));
        self.by_request.insert(key, font);
        font
    }

    /// The name of a font Core Text set a run in, the fallback fonts it picked among them.
    pub(crate) fn name(&mut self, font: &CTFont) -> FontId {
        let address = ptr::from_ref(font) as usize;
        if let Some(named) = self.by_address.get(&address) {
            return *named;
        }
        let named = self
            .fonts
            .iter()
            .position(|known| *known.0 == *font)
            .unwrap_or_else(|| {
                self.fonts.push(SharedFont(font.retain()));
                self.fonts.len() - 1
            });
        let named = FontId(named as u32);
        self.held_objects.push(SharedFont(font.retain()));
        self.by_address.insert(address, named);
        named
    }

    /// # Panics
    ///
    /// When the name was not given out by this library.
    pub(crate) fn font(&self, font: FontId) -> &CTFont {
        &self.fonts[font.0 as usize].0
    }
}
