use std::ffi::c_void;
use std::ptr::{self, NonNull};

use objc2_core_foundation::CGPoint;
use objc2_core_graphics::{
    CGBitmapContextCreate, CGColorSpace, CGContext, CGGlyph, CGImageAlphaInfo, CGImageByteOrderInfo, kCGColorSpaceSRGB,
};
use objc2_core_text::{CTFont, CTFontOrientation};

use crate::render::glyph_extent::GlyphExtent;

/// A glyph is rasterized at this many positions of the pen inside a device pixel.
pub(crate) const SUBPIXEL_PHASES: u8 = 4;
/// Anti-aliasing spills over the bounds a font reports for a glyph by less than a pixel.
const SPILL: i32 = 1;
const BITS_PER_CHANNEL: usize = 8;
const COLOR_CHANNELS: usize = 4;

/// Draws one glyph of a font, at the size of the font, into `pixels`: rows from the top, one
/// byte of coverage a pixel, or four of premultiplied blue, green, red and alpha for a color
/// font. Returns none for a glyph without pixels, such as a space.
///
/// The glyph is drawn in grayscale anti-aliasing with the pen at a fraction of a pixel and
/// without the stem darkening of font smoothing, as browsers draw `antialiased` text.
pub(crate) fn rasterize(
    font: &CTFont,
    glyph: CGGlyph,
    subpixel_phase: u8,
    in_color: bool,
    pixels: &mut Vec<u8>,
) -> Option<GlyphExtent> {
    let shift = f64::from(subpixel_phase) / f64::from(SUBPIXEL_PHASES);
    let mut glyph = glyph;
    // SAFETY: the pointer is to one glyph; a null pointer skips the rectangle of each glyph.
    let bounds = unsafe {
        font.bounding_rects_for_glyphs(
            CTFontOrientation::Default,
            NonNull::from(&mut glyph),
            ptr::null_mut(),
            1,
        )
    };
    if bounds.size.width <= 0.0 || bounds.size.height <= 0.0 {
        return None;
    }
    let left = (bounds.origin.x + shift).floor() as i32 - SPILL;
    let bottom = bounds.origin.y.floor() as i32 - SPILL;
    let right = (bounds.origin.x + bounds.size.width + shift).ceil() as i32 + SPILL;
    let top = (bounds.origin.y + bounds.size.height).ceil() as i32 + SPILL;
    let width = (right - left) as usize;
    let height = (top - bottom) as usize;
    let bytes_per_pixel = if in_color { COLOR_CHANNELS } else { 1 };
    pixels.clear();
    pixels.resize(width * height * bytes_per_pixel, 0);

    // SAFETY: the name is a constant of Core Graphics.
    let color_space = in_color.then(|| CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB })));
    let layout = if in_color {
        CGImageAlphaInfo::PremultipliedFirst.0 | CGImageByteOrderInfo::Order32Little.0
    } else {
        CGImageAlphaInfo::Only.0
    };
    // SAFETY: the buffer holds `height` rows of `width` pixels in the layout named.
    let context = unsafe {
        CGBitmapContextCreate(
            pixels.as_mut_ptr().cast::<c_void>(),
            width,
            height,
            BITS_PER_CHANNEL,
            width * bytes_per_pixel,
            color_space.flatten().as_deref(),
            layout,
        )
    }
    .expect("Core Graphics creates a bitmap context");
    let drawn = Some(&*context);
    CGContext::set_should_antialias(drawn, true);
    CGContext::set_allows_font_smoothing(drawn, false);
    CGContext::set_should_smooth_fonts(drawn, false);
    CGContext::set_allows_font_subpixel_positioning(drawn, true);
    CGContext::set_should_subpixel_position_fonts(drawn, true);
    CGContext::set_allows_font_subpixel_quantization(drawn, false);
    CGContext::set_should_subpixel_quantize_fonts(drawn, false);
    let mut pen = CGPoint {
        x: shift - f64::from(left),
        y: -f64::from(bottom),
    };
    // SAFETY: the pointers are to one glyph and one position.
    unsafe { font.draw_glyphs(NonNull::from(&mut glyph), NonNull::from(&mut pen), 1, &context) };
    Some(GlyphExtent {
        width: width as u32,
        height: height as u32,
        left,
        top,
    })
}
