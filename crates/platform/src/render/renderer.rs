use std::sync::{Mutex, PoisonError};

use dispatch2::DispatchData;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::{NSError, NSString};
use objc2_metal::{
    MTL4BlendState, MTL4CommandQueue, MTL4Compiler, MTL4CompilerDescriptor, MTL4FunctionDescriptor,
    MTL4LibraryFunctionDescriptor, MTL4PipelineDescriptor, MTL4RenderPipelineDescriptor, MTLBlendFactor,
    MTLCreateSystemDefaultDevice, MTLDevice, MTLGPUFamily, MTLLibrary, MTLPixelFormat, MTLRenderPipelineState,
};

use crate::renderer_error::RendererError;

const SHADERS: &[u8] = include_bytes!(env!("MOROK_SHADERS_METALLIB"));
const RECT_VERTEX_FUNCTION: &str = "rect_vertex";
const RECT_FRAGMENT_FUNCTION: &str = "rect_fragment";

/// Pipelines compiled at the same moment by separate Metal 4 compilers never come back from the
/// driver, so the renderers of a process compile theirs one after another.
static PIPELINE_COMPILATION: Mutex<()> = Mutex::new(());

/// Browsers composite in non-linear sRGB, so the frame buffer is not an `_srgb` format: colors
/// are blended as the sRGB values they are given in.
pub(crate) const PIXEL_FORMAT: MTLPixelFormat = MTLPixelFormat::BGRA8Unorm;

/// The GPU resources every surface of an application shares: the Metal 4 device, its command
/// queue and the pipeline that draws boxes. A clone is a handle to the same resources.
#[derive(Clone, Debug)]
pub struct Renderer {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    queue: Retained<ProtocolObject<dyn MTL4CommandQueue>>,
    rect_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
}

impl Renderer {
    /// The renderer of the default GPU of the system.
    pub fn new() -> Result<Self, RendererError> {
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
        let rect_pipeline = {
            let _one_at_a_time = PIPELINE_COMPILATION.lock().unwrap_or_else(PoisonError::into_inner);
            let compiler = device
                .newCompilerWithDescriptor_error(&MTL4CompilerDescriptor::new())
                .map_err(|error| metal_error("shader compiler", &error))?;
            rect_pipeline(&compiler, &library)?
        };
        Ok(Self {
            device,
            queue,
            rect_pipeline,
        })
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
}

/// The pipeline blends premultiplied colors over what is already drawn.
fn rect_pipeline(
    compiler: &ProtocolObject<dyn MTL4Compiler>,
    library: &ProtocolObject<dyn MTLLibrary>,
) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, RendererError> {
    let function = |name: &str| {
        let descriptor = MTL4LibraryFunctionDescriptor::new();
        descriptor.setName(Some(&NSString::from_str(name)));
        descriptor.setLibrary(Some(library));
        descriptor
    };
    let vertex_function = function(RECT_VERTEX_FUNCTION);
    let fragment_function = function(RECT_FRAGMENT_FUNCTION);
    let descriptor = MTL4RenderPipelineDescriptor::new();
    descriptor.setVertexFunctionDescriptor(Some(as_function_descriptor(&vertex_function)));
    descriptor.setFragmentFunctionDescriptor(Some(as_function_descriptor(&fragment_function)));
    // SAFETY: a pipeline has a color attachment at index 0.
    let attachment = unsafe { descriptor.colorAttachments().objectAtIndexedSubscript(0) };
    attachment.setPixelFormat(PIXEL_FORMAT);
    attachment.setBlendingState(MTL4BlendState::Enabled);
    attachment.setSourceRGBBlendFactor(MTLBlendFactor::One);
    attachment.setSourceAlphaBlendFactor(MTLBlendFactor::One);
    attachment.setDestinationRGBBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
    attachment.setDestinationAlphaBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
    let descriptor: &MTL4PipelineDescriptor = &descriptor;
    compiler
        .newRenderPipelineStateWithDescriptor_compilerTaskOptions_error(descriptor, None)
        .map_err(|error| metal_error("rectangle pipeline", &error))
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
