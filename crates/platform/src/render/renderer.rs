use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use dispatch2::DispatchData;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::{NSError, NSString};
use objc2_metal::{
    MTL4BlendState, MTL4CommandQueue, MTL4Compiler, MTL4CompilerDescriptor, MTL4FunctionDescriptor,
    MTL4LibraryFunctionDescriptor, MTL4PipelineDescriptor, MTL4RenderPipelineDescriptor, MTLBlendFactor,
    MTLCreateSystemDefaultDevice, MTLDevice, MTLGPUFamily, MTLLibrary, MTLPixelFormat, MTLRenderPipelineState,
};

use morok_core::{IconId, ImageId, Size};

use crate::atlas_statistics::AtlasStatistics;
use crate::image_statistics::ImageStatistics;
use crate::image_status::ImageStatus;
use crate::media::icon_library::IconLibrary;
use crate::media::image_store::{ImageState, ImageStore};
use crate::media::image_textures::ImageTextures;
use crate::render::atlases::Atlases;
use crate::renderer_error::RendererError;
use crate::renderer_options::RendererOptions;

const SHADERS: &[u8] = include_bytes!(env!("MOROK_SHADERS_METALLIB"));
const RECT_VERTEX_FUNCTION: &str = "rect_vertex";
const RECT_FRAGMENT_FUNCTION: &str = "rect_fragment";
const GLYPH_VERTEX_FUNCTION: &str = "glyph_vertex";
const GLYPH_FRAGMENT_FUNCTION: &str = "glyph_fragment";
const COMPOSITE_VERTEX_FUNCTION: &str = "composite_vertex";
const COMPOSITE_FRAGMENT_FUNCTION: &str = "composite_fragment";
const BLUR_VERTEX_FUNCTION: &str = "blur_vertex";
const BLUR_FRAGMENT_FUNCTION: &str = "blur_fragment";
const IMAGE_VERTEX_FUNCTION: &str = "image_vertex";
const IMAGE_FRAGMENT_FUNCTION: &str = "image_fragment";

/// Pipelines compiled at the same moment by separate Metal 4 compilers never come back from the
/// driver, so the renderers of a process compile theirs one after another.
static PIPELINE_COMPILATION: Mutex<()> = Mutex::new(());

/// Browsers composite in non-linear sRGB, so the frame buffer is not an `_srgb` format: colors
/// are blended as the sRGB values they are given in.
pub(crate) const PIXEL_FORMAT: MTLPixelFormat = MTLPixelFormat::BGRA8Unorm;

/// The GPU resources every surface of an application shares: the Metal 4 device, its command
/// queue, the pipelines that draw boxes, glyphs and the pictures of groups, and the glyph
/// atlases. A clone is a handle to the same resources.
#[derive(Clone, Debug)]
pub struct Renderer {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    queue: Retained<ProtocolObject<dyn MTL4CommandQueue>>,
    rect_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    glyph_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    /// Lays the picture of a group over a target; it reads the target and writes the result,
    /// so it blends nothing itself.
    composite_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    /// One pass of a Gaussian blur into a cleared target.
    blur_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    /// The picture of an image, sampled with filtering, over a quad of any size.
    image_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    glyph_atlases: Arc<Mutex<Atlases>>,
    icons: Arc<Mutex<IconLibrary>>,
    images: Arc<Mutex<ImageStore>>,
    image_textures: Arc<Mutex<ImageTextures>>,
}

impl Renderer {
    /// The renderer of the default GPU of the system.
    pub fn new() -> Result<Self, RendererError> {
        Self::with_options(&RendererOptions::default())
    }

    pub fn with_options(options: &RendererOptions) -> Result<Self, RendererError> {
        let device = MTLCreateSystemDefaultDevice().ok_or(RendererError::NoDevice)?;
        if !device.supportsFamily(MTLGPUFamily::Metal4) {
            return Err(RendererError::Metal4Unsupported);
        }
        let queue = device.newMTL4CommandQueue().ok_or(RendererError::Metal {
            object: "command queue",
            reason: String::from("the device returned none"),
        })?;
        let library = device
            .newLibraryWithData_error(&DispatchData::from_static_bytes(SHADERS))
            .map_err(|error| metal_error("shader library", &error))?;
        let (rect_pipeline, glyph_pipeline, composite_pipeline, blur_pipeline, image_pipeline) = {
            let _one_at_a_time = PIPELINE_COMPILATION.lock().unwrap_or_else(PoisonError::into_inner);
            let compiler = device
                .newCompilerWithDescriptor_error(&MTL4CompilerDescriptor::new())
                .map_err(|error| metal_error("shader compiler", &error))?;
            (
                pipeline(
                    &compiler,
                    &library,
                    RECT_VERTEX_FUNCTION,
                    RECT_FRAGMENT_FUNCTION,
                    Blending::Over,
                )?,
                pipeline(
                    &compiler,
                    &library,
                    GLYPH_VERTEX_FUNCTION,
                    GLYPH_FRAGMENT_FUNCTION,
                    Blending::Over,
                )?,
                pipeline(
                    &compiler,
                    &library,
                    COMPOSITE_VERTEX_FUNCTION,
                    COMPOSITE_FRAGMENT_FUNCTION,
                    Blending::Replace,
                )?,
                pipeline(
                    &compiler,
                    &library,
                    BLUR_VERTEX_FUNCTION,
                    BLUR_FRAGMENT_FUNCTION,
                    Blending::Replace,
                )?,
                pipeline(
                    &compiler,
                    &library,
                    IMAGE_VERTEX_FUNCTION,
                    IMAGE_FRAGMENT_FUNCTION,
                    Blending::Over,
                )?,
            )
        };
        Ok(Self {
            glyph_atlases: Arc::new(Mutex::new(Atlases::new(&device, options))),
            icons: Arc::new(Mutex::new(IconLibrary::default())),
            images: Arc::new(Mutex::new(ImageStore::default())),
            image_textures: Arc::new(Mutex::new(ImageTextures::new(&device, options.image_byte_budget))),
            device,
            queue,
            rect_pipeline,
            glyph_pipeline,
            composite_pipeline,
            blur_pipeline,
            image_pipeline,
        })
    }

    /// Takes an icon as SVG. `currentColor` in it is the `color` of the element that shows
    /// the icon; an icon painted in nothing but `currentColor` is tinted by the shader.
    pub fn add_icon(&self, svg: &str) -> Result<IconId, RendererError> {
        self.icons()
            .add(svg)
            .map_err(|error| RendererError::UnreadableIcon(error.to_string()))
    }

    /// Takes an encoded image — PNG, JPEG, HEIC or any format the system reads — and decodes
    /// it on a thread of its own; [`image_status`](Self::image_status) tells when it is ready.
    pub fn add_image(&self, encoded: Vec<u8>) -> ImageId {
        self.images().add(encoded)
    }

    /// # Panics
    ///
    /// When the renderer never gave out the id.
    pub fn image_status(&self, image: ImageId) -> ImageStatus {
        match self.images().state(image).expect("the renderer gave the image id out") {
            ImageState::Decoding => ImageStatus::Decoding,
            ImageState::Unreadable => ImageStatus::Unreadable,
            ImageState::Decoded(decoded) => ImageStatus::Decoded(Size {
                width: decoded.width as f32,
                height: decoded.height as f32,
            }),
        }
    }

    /// What the textures of large images take and have done so far.
    pub fn image_statistics(&self) -> ImageStatistics {
        let textures = self.image_textures();
        ImageStatistics {
            texture_bytes: textures.bytes(),
            uploads: textures.uploads(),
            evictions: textures.evictions(),
        }
    }

    pub(crate) fn icons(&self) -> MutexGuard<'_, IconLibrary> {
        self.icons.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn images(&self) -> MutexGuard<'_, ImageStore> {
        self.images.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn image_textures(&self) -> MutexGuard<'_, ImageTextures> {
        self.image_textures.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn image_pipeline(&self) -> &ProtocolObject<dyn MTLRenderPipelineState> {
        &self.image_pipeline
    }

    /// How full the glyph atlases are and what they have done so far.
    pub fn atlas_statistics(&self) -> AtlasStatistics {
        self.glyph_atlases().statistics()
    }

    pub(crate) fn glyph_atlases(&self) -> MutexGuard<'_, Atlases> {
        self.glyph_atlases.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn glyph_pipeline(&self) -> &ProtocolObject<dyn MTLRenderPipelineState> {
        &self.glyph_pipeline
    }

    pub(crate) fn device(&self) -> &Retained<ProtocolObject<dyn MTLDevice>> {
        &self.device
    }

    pub(crate) fn queue(&self) -> &Retained<ProtocolObject<dyn MTL4CommandQueue>> {
        &self.queue
    }

    pub(crate) fn rect_pipeline(&self) -> &ProtocolObject<dyn MTLRenderPipelineState> {
        &self.rect_pipeline
    }

    pub(crate) fn composite_pipeline(&self) -> &ProtocolObject<dyn MTLRenderPipelineState> {
        &self.composite_pipeline
    }

    pub(crate) fn blur_pipeline(&self) -> &ProtocolObject<dyn MTLRenderPipelineState> {
        &self.blur_pipeline
    }
}

/// How the fragments of a pipeline meet what the target already holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Blending {
    /// Premultiplied colors laid over the target.
    Over,
    /// The fragment is the result: the shader read the target itself, or the target is cleared.
    Replace,
}

/// A pipeline of the two named functions.
fn pipeline(
    compiler: &ProtocolObject<dyn MTL4Compiler>,
    library: &ProtocolObject<dyn MTLLibrary>,
    vertex_function: &str,
    fragment_function: &str,
    blending: Blending,
) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, RendererError> {
    let function = |name: &str| {
        let descriptor = MTL4LibraryFunctionDescriptor::new();
        descriptor.setName(Some(&NSString::from_str(name)));
        descriptor.setLibrary(Some(library));
        descriptor
    };
    let vertex_function = function(vertex_function);
    let fragment_function = function(fragment_function);
    let descriptor = MTL4RenderPipelineDescriptor::new();
    descriptor.setVertexFunctionDescriptor(Some(as_function_descriptor(&vertex_function)));
    descriptor.setFragmentFunctionDescriptor(Some(as_function_descriptor(&fragment_function)));
    // SAFETY: a pipeline has a color attachment at index 0.
    let attachment = unsafe { descriptor.colorAttachments().objectAtIndexedSubscript(0) };
    attachment.setPixelFormat(PIXEL_FORMAT);
    match blending {
        Blending::Over => {
            attachment.setBlendingState(MTL4BlendState::Enabled);
            attachment.setSourceRGBBlendFactor(MTLBlendFactor::One);
            attachment.setSourceAlphaBlendFactor(MTLBlendFactor::One);
            attachment.setDestinationRGBBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
            attachment.setDestinationAlphaBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
        }
        Blending::Replace => attachment.setBlendingState(MTL4BlendState::Disabled),
    }
    let descriptor: &MTL4PipelineDescriptor = &descriptor;
    compiler
        .newRenderPipelineStateWithDescriptor_compilerTaskOptions_error(descriptor, None)
        .map_err(|error| metal_error("render pipeline", &error))
}

fn as_function_descriptor(function: &MTL4LibraryFunctionDescriptor) -> &MTL4FunctionDescriptor {
    function
}

fn metal_error(object: &'static str, error: &NSError) -> RendererError {
    RendererError::Metal {
        object,
        reason: error.localizedDescription().to_string(),
    }
}
