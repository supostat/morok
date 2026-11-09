use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTL4ArgumentTable, MTL4ArgumentTableDescriptor, MTL4CommandAllocator, MTLDevice, MTLResidencySet, MTLTexture,
};

use crate::render::blur_uniforms::BlurUniforms;
use crate::render::clip_shape::ClipShape;
use crate::render::composite_instance::CompositeInstance;
use crate::render::filter_entry::FilterEntry;
use crate::render::frame_command::FrameCommand;
use crate::render::frame_uniforms::FrameUniforms;
use crate::render::glyph_instance::GlyphInstance;
use crate::render::gpu_buffer::GpuBuffer;
use crate::render::gradient_stop_entry::GradientStopEntry;
use crate::render::group_pass::GroupPass;
use crate::render::rect_instance::RectInstance;
use crate::render::texture_pool::TexturePool;
use crate::render::transform_entry::TransformEntry;

const INSTANCES_BINDING: usize = 0;
const UNIFORMS_BINDING: usize = 1;
const GRADIENT_STOPS_BINDING: usize = 2;
const CLIPS_BINDING: usize = 3;
const GLYPH_INSTANCES_BINDING: usize = 4;
const TRANSFORMS_BINDING: usize = 5;
const COMPOSITE_INSTANCES_BINDING: usize = 6;
const FILTERS_BINDING: usize = 7;
const BLUR_UNIFORMS_BINDING: usize = 8;
const BUFFER_BINDINGS: usize = 9;
/// The atlas page of a glyph batch, the picture of a composite or the source of a blur pass.
const TEXTURE_BINDING: usize = 0;
const TEXTURE_BINDINGS: usize = 1;

const INITIAL_INSTANCES: usize = 256;
const INITIAL_GLYPH_INSTANCES: usize = 1024;
const INITIAL_GRADIENT_STOPS: usize = 32;
const INITIAL_CLIPS: usize = 16;
const INITIAL_TRANSFORMS: usize = 8;
const INITIAL_COMPOSITES: usize = 8;
const INITIAL_FILTERS: usize = 16;
const INITIAL_PASSES: usize = 4;

/// Everything one frame needs from the moment it is encoded until the GPU has drawn it. A
/// surface holds as many of these as it may have frames in flight.
#[derive(Debug)]
pub(crate) struct FrameResources {
    pub(crate) instances: GpuBuffer<RectInstance>,
    pub(crate) glyph_instances: GpuBuffer<GlyphInstance>,
    pub(crate) gradient_stops: GpuBuffer<GradientStopEntry>,
    pub(crate) clips: GpuBuffer<ClipShape>,
    pub(crate) transforms: GpuBuffer<TransformEntry>,
    pub(crate) composites: GpuBuffer<CompositeInstance>,
    pub(crate) filters: GpuBuffer<FilterEntry>,
    /// One entry per target drawn into: the surface, then the picture of every group.
    pub(crate) uniforms: GpuBuffer<FrameUniforms>,
    pub(crate) blur_passes: GpuBuffer<BlurUniforms>,
    pub(crate) commands: Vec<FrameCommand>,
    pub(crate) groups: Vec<GroupPass>,
    pub(crate) textures: TexturePool,
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
            transforms: GpuBuffer::with_capacity(device, residency_set, INITIAL_TRANSFORMS),
            composites: GpuBuffer::with_capacity(device, residency_set, INITIAL_COMPOSITES),
            filters: GpuBuffer::with_capacity(device, residency_set, INITIAL_FILTERS),
            uniforms: GpuBuffer::with_capacity(device, residency_set, INITIAL_PASSES),
            blur_passes: GpuBuffer::with_capacity(device, residency_set, INITIAL_PASSES),
            commands: Vec::new(),
            groups: Vec::new(),
            textures: TexturePool::default(),
            argument_table,
            allocator,
            last_frame: 0,
        }
    }

    /// Empties the buffers for a new frame. The GPU must have finished the previous one. The
    /// first transform is the identity, which every instance outside a transformed group uses.
    pub(crate) fn begin_frame(&mut self) {
        self.allocator.reset();
        self.instances.clear();
        self.glyph_instances.clear();
        self.gradient_stops.clear();
        self.clips.clear();
        self.transforms.clear();
        self.transforms.push(TransformEntry::IDENTITY);
        self.composites.clear();
        self.filters.clear();
        self.uniforms.clear();
        self.blur_passes.clear();
        self.commands.clear();
        self.groups.clear();
        self.textures.begin_frame();
    }

    /// Points the shaders at the buffers as they are now: a buffer that grew has moved.
    pub(crate) fn bind_buffers(&self) {
        let bindings = [
            (INSTANCES_BINDING, self.instances.gpu_address()),
            (UNIFORMS_BINDING, self.uniforms.gpu_address()),
            (GRADIENT_STOPS_BINDING, self.gradient_stops.gpu_address()),
            (CLIPS_BINDING, self.clips.gpu_address()),
            (GLYPH_INSTANCES_BINDING, self.glyph_instances.gpu_address()),
            (TRANSFORMS_BINDING, self.transforms.gpu_address()),
            (COMPOSITE_INSTANCES_BINDING, self.composites.gpu_address()),
            (FILTERS_BINDING, self.filters.gpu_address()),
            (BLUR_UNIFORMS_BINDING, self.blur_passes.gpu_address()),
        ];
        for (binding, address) in bindings {
            // SAFETY: the binding is below the bind count of the table and the address is that
            // of a live buffer of the residency set.
            unsafe { self.argument_table.setAddress_atIndex(address, binding) };
        }
    }

    /// Points the shaders at the uniforms of the target about to be drawn into.
    pub(crate) fn bind_uniforms(&self, pass: usize) {
        // SAFETY: the binding is below the bind count of the table and the address is inside a
        // live buffer of the residency set.
        unsafe {
            self.argument_table
                .setAddress_atIndex(self.uniforms.gpu_address_of(pass), UNIFORMS_BINDING);
        }
    }

    /// Points the composite shader at the composites as they are now: the buffer grows while
    /// the frame is drawn.
    pub(crate) fn bind_composites(&self) {
        // SAFETY: as for the uniforms.
        unsafe {
            self.argument_table
                .setAddress_atIndex(self.composites.gpu_address(), COMPOSITE_INSTANCES_BINDING);
        }
    }

    /// Points the blur shader at the uniforms of its next pass.
    pub(crate) fn bind_blur_pass(&self, pass: usize) {
        // SAFETY: as for the uniforms.
        unsafe {
            self.argument_table
                .setAddress_atIndex(self.blur_passes.gpu_address_of(pass), BLUR_UNIFORMS_BINDING);
        }
    }

    /// Points the shaders at the texture the next draw reads: an atlas page, a picture or the
    /// source of a blur.
    pub(crate) fn bind_texture(&self, texture: &ProtocolObject<dyn MTLTexture>) {
        // SAFETY: the binding is below the texture bind count of the table and the texture is a
        // live one of a residency set of the frame.
        unsafe {
            self.argument_table
                .setTexture_atIndex(texture.gpuResourceID(), TEXTURE_BINDING);
        }
    }
}
