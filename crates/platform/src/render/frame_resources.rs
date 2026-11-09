use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTL4ArgumentTable, MTL4ArgumentTableDescriptor, MTL4CommandAllocator, MTLDevice, MTLResidencySet, MTLTexture,
};

use crate::render::draw_batch::DrawBatch;
use crate::render::frame_uniforms::FrameUniforms;
use crate::render::glyph_instance::GlyphInstance;
use crate::render::gpu_buffer::GpuBuffer;
use crate::render::gradient_stop_entry::GradientStopEntry;
use crate::render::rect_instance::RectInstance;
use crate::render::rounded_rect::RoundedRect;

const INSTANCES_BINDING: usize = 0;
const UNIFORMS_BINDING: usize = 1;
const GRADIENT_STOPS_BINDING: usize = 2;
const CLIPS_BINDING: usize = 3;
const GLYPH_INSTANCES_BINDING: usize = 4;
const BUFFER_BINDINGS: usize = 5;
const ATLAS_PAGE_BINDING: usize = 0;
const TEXTURE_BINDINGS: usize = 1;

const INITIAL_INSTANCES: usize = 256;
const INITIAL_GLYPH_INSTANCES: usize = 1024;
const INITIAL_GRADIENT_STOPS: usize = 32;
const INITIAL_CLIPS: usize = 16;

/// Everything one frame needs from the moment it is encoded until the GPU has drawn it. A
/// surface holds as many of these as it may have frames in flight.
#[derive(Debug)]
pub(crate) struct FrameResources {
    pub(crate) instances: GpuBuffer<RectInstance>,
    pub(crate) glyph_instances: GpuBuffer<GlyphInstance>,
    pub(crate) gradient_stops: GpuBuffer<GradientStopEntry>,
    pub(crate) clips: GpuBuffer<RoundedRect>,
    pub(crate) uniforms: GpuBuffer<FrameUniforms>,
    pub(crate) draws: Vec<DrawBatch>,
    pub(crate) argument_table: Retained<ProtocolObject<dyn MTL4ArgumentTable>>,
    pub(crate) allocator: Retained<ProtocolObject<dyn MTL4CommandAllocator>>,
    /// The number of the last frame drawn from these resources; they are free once the GPU
    /// finished it.
    pub(crate) last_frame: u64,
}

impl FrameResources {
    pub(crate) fn new(
        device: &Retained<ProtocolObject<dyn MTLDevice>>,
        residency_set: &Retained<ProtocolObject<dyn MTLResidencySet>>,
    ) -> Self {
        let table_descriptor = MTL4ArgumentTableDescriptor::new();
        table_descriptor.setMaxBufferBindCount(BUFFER_BINDINGS);
        table_descriptor.setMaxTextureBindCount(TEXTURE_BINDINGS);
        let argument_table = device
            .newArgumentTableWithDescriptor_error(&table_descriptor)
            .unwrap_or_else(|error| panic!("Metal creates an argument table: {}", error.localizedDescription()));
        let allocator = device.newCommandAllocator().expect("Metal creates a command allocator");
        Self {
            instances: GpuBuffer::with_capacity(device, residency_set, INITIAL_INSTANCES),
            glyph_instances: GpuBuffer::with_capacity(device, residency_set, INITIAL_GLYPH_INSTANCES),
            gradient_stops: GpuBuffer::with_capacity(device, residency_set, INITIAL_GRADIENT_STOPS),
            clips: GpuBuffer::with_capacity(device, residency_set, INITIAL_CLIPS),
            uniforms: GpuBuffer::with_capacity(device, residency_set, 1),
            draws: Vec::new(),
            argument_table,
            allocator,
            last_frame: 0,
        }
    }

    /// Empties the buffers for a new frame. The GPU must have finished the previous one.
    pub(crate) fn begin_frame(&mut self, uniforms: FrameUniforms) {
        self.allocator.reset();
        self.instances.clear();
        self.glyph_instances.clear();
        self.gradient_stops.clear();
        self.clips.clear();
        self.draws.clear();
        self.uniforms.clear();
        self.uniforms.push(uniforms);
    }

    /// Points the shaders at the buffers as they are now: a buffer that grew has moved.
    pub(crate) fn bind_buffers(&self) {
        let bindings = [
            (INSTANCES_BINDING, self.instances.gpu_address()),
            (UNIFORMS_BINDING, self.uniforms.gpu_address()),
            (GRADIENT_STOPS_BINDING, self.gradient_stops.gpu_address()),
            (CLIPS_BINDING, self.clips.gpu_address()),
            (GLYPH_INSTANCES_BINDING, self.glyph_instances.gpu_address()),
        ];
        for (binding, address) in bindings {
            // SAFETY: the binding is below the bind count of the table and the address is that
            // of a live buffer of the residency set.
            unsafe { self.argument_table.setAddress_atIndex(address, binding) };
        }
    }

    /// Points the glyph shader at the atlas page the next glyphs are drawn from.
    pub(crate) fn bind_atlas_page(&self, page: &ProtocolObject<dyn MTLTexture>) {
        // SAFETY: the binding is below the texture bind count of the table and the page is a
        // live texture of the residency set of the frame.
        unsafe {
            self.argument_table
                .setTexture_atIndex(page.gpuResourceID(), ATLAS_PAGE_BINDING);
        }
    }
}
