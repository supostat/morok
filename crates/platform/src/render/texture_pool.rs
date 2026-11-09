use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLAllocation, MTLDevice, MTLResidencySet, MTLStorageMode, MTLTexture, MTLTextureDescriptor, MTLTextureUsage,
};

use crate::render::renderer::PIXEL_FORMAT;

/// A texture that pictures of groups are drawn into; a frame hands it back when it is done.
#[derive(Debug)]
struct PooledTexture {
    texture: Retained<ProtocolObject<dyn MTLTexture>>,
    width: usize,
    height: usize,
    is_taken: bool,
}

/// The textures the groups of a frame draw their pictures into. A texture outlives the frame
/// and serves the next one that needs its size, so a steady scene allocates none.
#[derive(Debug, Default)]
pub(crate) struct TexturePool {
    textures: Vec<PooledTexture>,
}

impl TexturePool {
    /// Hands every texture back for the next frame. The GPU must have finished the previous one.
    pub(crate) fn begin_frame(&mut self) {
        for texture in &mut self.textures {
            texture.is_taken = false;
        }
    }

    /// A texture of at least `width` by `height` texels, the smallest free one that fits, or a
    /// new one of exactly that size.
    pub(crate) fn acquire(
        &mut self,
        device: &ProtocolObject<dyn MTLDevice>,
        residency_set: &ProtocolObject<dyn MTLResidencySet>,
        width: usize,
        height: usize,
    ) -> usize {
        let fitting = self
            .textures
            .iter()
            .enumerate()
            .filter(|(_, texture)| !texture.is_taken && texture.width >= width && texture.height >= height)
            .min_by_key(|(_, texture)| texture.width * texture.height)
            .map(|(position, _)| position);
        let position = fitting.unwrap_or_else(|| {
            // SAFETY: the format is a color format and the size is not zero.
            let descriptor = unsafe {
                MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
                    PIXEL_FORMAT,
                    width.max(1),
                    height.max(1),
                    false,
                )
            };
            descriptor.setStorageMode(MTLStorageMode::Private);
            descriptor.setUsage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
            let texture = device
                .newTextureWithDescriptor(&descriptor)
                .expect("Metal creates a texture for a group");
            residency_set.addAllocation(ProtocolObject::<dyn MTLAllocation>::from_ref(&*texture));
            residency_set.commit();
            self.textures.push(PooledTexture {
                texture,
                width: width.max(1),
                height: height.max(1),
                is_taken: false,
            });
            self.textures.len() - 1
        });
        self.textures[position].is_taken = true;
        position
    }

    pub(crate) fn texture(&self, position: usize) -> &ProtocolObject<dyn MTLTexture> {
        &self.textures[position].texture
    }
}
