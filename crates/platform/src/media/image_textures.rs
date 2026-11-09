use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::NonNull;

use morok_core::ImageId;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLAllocation, MTLDevice, MTLOrigin, MTLRegion, MTLResidencySet, MTLResidencySetDescriptor, MTLSize,
    MTLStorageMode, MTLTexture, MTLTextureDescriptor, MTLTextureUsage,
};

use crate::media::decoded_image::DecodedImage;
use crate::render::renderer::PIXEL_FORMAT;
use crate::render::shared_residency_set::SharedResidencySet;
use crate::render::shared_texture::SharedTexture;

/// Frames a surface may have on their way to the screen: a dropped texture lives this many
/// frames longer, since one of them may still read it.
const FRAMES_IN_FLIGHT: u64 = 3;

#[derive(Debug)]
struct ImageTexture {
    texture: SharedTexture,
    bytes: usize,
    /// The last frame that drew the image.
    last_used: u64,
}

/// A texture dropped from the budget that a frame in flight may still read.
#[derive(Debug)]
struct RetiredTexture {
    texture: SharedTexture,
    /// The frame it was dropped in.
    dropped: u64,
}

/// The textures of the images too large for the color atlas, kept under a byte budget: when
/// a new one does not fit, the images no frame has drawn for the longest are dropped and are
/// uploaded again when next drawn. The textures live in a residency set of their own that
/// every command buffer of the renderer uses.
#[derive(Debug)]
pub(crate) struct ImageTextures {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    residency_set: SharedResidencySet,
    budget: usize,
    textures: HashMap<ImageId, ImageTexture>,
    retired: Vec<RetiredTexture>,
    bytes: usize,
    frame: u64,
    uploads: u64,
    evictions: u64,
}

impl ImageTextures {
    pub(crate) fn new(device: &Retained<ProtocolObject<dyn MTLDevice>>, budget: usize) -> Self {
        let residency_set = device
            .newResidencySetWithDescriptor_error(&MTLResidencySetDescriptor::new())
            .unwrap_or_else(|error| panic!("Metal creates a residency set: {}", error.localizedDescription()));
        Self {
            device: device.clone(),
            residency_set: SharedResidencySet(residency_set),
            budget,
            textures: HashMap::new(),
            retired: Vec::new(),
            bytes: 0,
            frame: 0,
            uploads: 0,
            evictions: 0,
        }
    }

    /// From here on the textures a frame draws are in use: they are not dropped before the
    /// next frame begins. Textures dropped long enough ago leave the GPU now.
    pub(crate) fn begin_frame(&mut self) {
        self.frame += 1;
        let frame = self.frame;
        let mut released = false;
        self.retired.retain(|retired| {
            let is_in_flight = retired.dropped + FRAMES_IN_FLIGHT >= frame;
            if !is_in_flight {
                self.residency_set
                    .0
                    .removeAllocation(ProtocolObject::<dyn MTLAllocation>::from_ref(&*retired.texture.0));
                released = true;
            }
            is_in_flight
        });
        if released {
            self.residency_set.0.commit();
        }
    }

    /// The set every command buffer that may draw an image uses.
    pub(crate) fn residency_set(&self) -> &ProtocolObject<dyn MTLResidencySet> {
        &self.residency_set.0
    }

    /// The texture of the image, uploaded now when no frame holds it; what the budget cannot
    /// hold is dropped first.
    pub(crate) fn texture(&mut self, image: ImageId, decoded: &DecodedImage) -> &ProtocolObject<dyn MTLTexture> {
        if !self.textures.contains_key(&image) {
            self.make_room(decoded.bytes());
            self.upload(image, decoded);
        }
        let held = self.textures.get_mut(&image).expect("the image was uploaded");
        held.last_used = self.frame;
        &held.texture.0
    }

    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }

    pub(crate) fn uploads(&self) -> u64 {
        self.uploads
    }

    pub(crate) fn evictions(&self) -> u64 {
        self.evictions
    }

    fn make_room(&mut self, bytes: usize) {
        while self.bytes + bytes > self.budget {
            let stale = self
                .textures
                .iter()
                .filter(|(_, texture)| texture.last_used < self.frame)
                .min_by_key(|(_, texture)| texture.last_used)
                .map(|(image, _)| *image);
            let Some(image) = stale else {
                return;
            };
            let dropped = self.textures.remove(&image).expect("the image was found");
            self.bytes -= dropped.bytes;
            self.evictions += 1;
            self.retired.push(RetiredTexture {
                texture: dropped.texture,
                dropped: self.frame,
            });
        }
    }

    fn upload(&mut self, image: ImageId, decoded: &DecodedImage) {
        let width = decoded.width as usize;
        let height = decoded.height as usize;
        // SAFETY: the format is a color format and the size is not zero.
        let descriptor = unsafe {
            MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
                PIXEL_FORMAT,
                width,
                height,
                false,
            )
        };
        descriptor.setStorageMode(MTLStorageMode::Shared);
        descriptor.setUsage(MTLTextureUsage::ShaderRead);
        let texture = self
            .device
            .newTextureWithDescriptor(&descriptor)
            .expect("Metal creates a texture for an image");
        let region = MTLRegion {
            origin: MTLOrigin { x: 0, y: 0, z: 0 },
            size: MTLSize {
                width,
                height,
                depth: 1,
            },
        };
        // SAFETY: the pixels are `height` rows of `width` four-byte texels and the region is
        // the whole texture.
        unsafe {
            texture.replaceRegion_mipmapLevel_withBytes_bytesPerRow(
                region,
                0,
                NonNull::new(decoded.pixels.as_ptr().cast_mut().cast::<c_void>()).expect("a vector is not null"),
                width * 4,
            );
        }
        self.residency_set
            .0
            .addAllocation(ProtocolObject::<dyn MTLAllocation>::from_ref(&*texture));
        self.residency_set.0.commit();
        self.bytes += decoded.bytes();
        self.uploads += 1;
        self.textures.insert(
            image,
            ImageTexture {
                texture: SharedTexture(texture),
                bytes: decoded.bytes(),
                last_used: self.frame,
            },
        );
    }
}
