use std::ffi::c_void;
use std::ptr::NonNull;

use morok_core::{DisplayList, Size};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLDevice, MTLOrigin, MTLRegion, MTLSize, MTLStorageMode, MTLTexture, MTLTextureDescriptor, MTLTextureUsage,
};

use crate::frame_image::FrameImage;
use crate::render::frame_renderer::FrameRenderer;
use crate::render::renderer::{PIXEL_FORMAT, Renderer};
use crate::render::target_geometry::TargetGeometry;

const BYTES_PER_PIXEL: usize = 4;
const BLUE: usize = 0;
const RED: usize = 2;

/// A surface that is not on screen: display lists are drawn into a texture and read back.
#[derive(Debug)]
pub struct OffscreenTarget {
    frame_renderer: FrameRenderer,
    texture: Retained<ProtocolObject<dyn MTLTexture>>,
    geometry: TargetGeometry,
}

impl OffscreenTarget {
    /// A target for a surface of `viewport` CSS pixels, drawn with `scale` device pixels to a
    /// CSS pixel.
    ///
    /// # Panics
    ///
    /// When the surface has no pixel.
    pub fn new(renderer: &Renderer, viewport: Size, scale: f32) -> Self {
        let geometry = TargetGeometry::new(viewport, scale);
        assert!(
            !geometry.is_empty(),
            "an offscreen target of {viewport:?} at scale {scale} has no pixel"
        );
        // SAFETY: the format is a color format and the size is not zero.
        let descriptor = unsafe {
            MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
                PIXEL_FORMAT,
                geometry.pixel_width,
                geometry.pixel_height,
                false,
            )
        };
        descriptor.setStorageMode(MTLStorageMode::Shared);
        descriptor.setUsage(MTLTextureUsage::RenderTarget);
        let texture = renderer
            .device()
            .newTextureWithDescriptor(&descriptor)
            .expect("Metal creates an offscreen texture");
        let frame_renderer = FrameRenderer::new(renderer, 1);
        frame_renderer.keep_resident(&texture);
        Self {
            frame_renderer,
            texture,
            geometry,
        }
    }

    /// Draws the display list on a transparent surface and returns without waiting for the GPU.
    pub fn draw(&mut self, display_list: &DisplayList) {
        self.frame_renderer
            .render(display_list, &self.geometry, &self.texture, None);
    }

    /// Blocks until the GPU has drawn every display list handed to it.
    pub fn wait_until_drawn(&self) {
        self.frame_renderer.wait_until_rendered();
    }

    /// Draws the display list on a transparent surface and returns the frame.
    pub fn render(&mut self, display_list: &DisplayList) -> FrameImage {
        self.draw(display_list);
        self.wait_until_drawn();
        let TargetGeometry {
            pixel_width: width,
            pixel_height: height,
            ..
        } = self.geometry;
        let mut pixels = vec![0_u8; width * height * BYTES_PER_PIXEL];
        // SAFETY: the vector holds `height` rows of `width` four-byte pixels, the region is the
        // whole texture and the GPU has finished drawing into it.
        unsafe {
            self.texture.getBytes_bytesPerRow_fromRegion_mipmapLevel(
                NonNull::new(pixels.as_mut_ptr().cast::<c_void>()).expect("a vector is not null"),
                width * BYTES_PER_PIXEL,
                MTLRegion {
                    origin: MTLOrigin { x: 0, y: 0, z: 0 },
                    size: MTLSize {
                        width,
                        height,
                        depth: 1,
                    },
                },
                0,
            );
        }
        for pixel in pixels.chunks_exact_mut(BYTES_PER_PIXEL) {
            pixel.swap(BLUE, RED);
        }
        FrameImage { width, height, pixels }
    }
}
