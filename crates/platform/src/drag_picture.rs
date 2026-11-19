use std::ffi::c_void;

use morok_core::{DisplayList, Rect};
use objc2::AnyThread;
use objc2::rc::Retained;
use objc2_app_kit::NSImage;
use objc2_core_graphics::{
    CGBitmapContextCreate, CGBitmapContextCreateImage, CGColorSpace, CGImageAlphaInfo, kCGColorSpaceSRGB,
};
use objc2_foundation::NSSize;

use crate::render::offscreen_target::OffscreenTarget;
use crate::render::pixel_rect::PixelRect;
use crate::render::renderer::Renderer;
use crate::render::target_geometry::TargetGeometry;

const BITS_PER_CHANNEL: usize = 8;
const BYTES_PER_PIXEL: usize = 4;

/// The picture a drag of the system shows under the pointer: what a surface drew of the source
/// of the drag, drawn again off screen with the pixels of the surface and cut to the part of
/// the surface it covers.
#[derive(Debug)]
pub(crate) struct DragPicture {
    pub(crate) image: Retained<NSImage>,
    /// Where the picture lies on the surface, in CSS pixels.
    pub(crate) frame: Rect,
}

impl DragPicture {
    /// Draws `picture`, the display list of the source alone, which covers `covered` of a
    /// surface of `geometry`. Returns none when it covers no pixel of the surface.
    pub(crate) fn render(
        renderer: &Renderer,
        picture: &DisplayList,
        covered: &Rect,
        geometry: &TargetGeometry,
    ) -> Option<Self> {
        if geometry.is_empty() {
            return None;
        }
        let shown = PixelRect::covering(covered, geometry.scale).intersection(&geometry.pixels());
        if shown.is_empty() {
            return None;
        }
        let surface = OffscreenTarget::new(renderer, geometry.viewport, geometry.scale).render(picture);
        let row_length = shown.width * BYTES_PER_PIXEL;
        let mut pixels = Vec::with_capacity(shown.height * row_length);
        for row in shown.y..shown.y + shown.height {
            let start = (row * surface.width + shown.x) * BYTES_PER_PIXEL;
            pixels.extend_from_slice(&surface.pixels[start..start + row_length]);
        }
        // SAFETY: the name is a constant of Core Graphics.
        let color_space = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB }));
        // SAFETY: the buffer holds `height` rows of `width` pixels of red, green, blue and
        // alpha, premultiplied, and outlives the context.
        let context = unsafe {
            CGBitmapContextCreate(
                pixels.as_mut_ptr().cast::<c_void>(),
                shown.width,
                shown.height,
                BITS_PER_CHANNEL,
                row_length,
                color_space.as_deref(),
                CGImageAlphaInfo::PremultipliedLast.0,
            )
        }?;
        let bitmap = CGBitmapContextCreateImage(Some(&context))?;
        let scale = f64::from(geometry.scale);
        let size = NSSize::new(shown.width as f64 / scale, shown.height as f64 / scale);
        Some(Self {
            image: NSImage::initWithCGImage_size(NSImage::alloc(), &bitmap, size),
            frame: Rect::new(
                shown.x as f32 / geometry.scale,
                shown.y as f32 / geometry.scale,
                shown.width as f32 / geometry.scale,
                shown.height as f32 / geometry.scale,
            ),
        })
    }
}
