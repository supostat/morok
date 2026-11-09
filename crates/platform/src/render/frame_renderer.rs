use std::ptr::NonNull;

use morok_core::DisplayList;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTL4CommandBuffer, MTL4CommandEncoder, MTL4CommandQueue, MTL4RenderCommandEncoder, MTL4RenderPassDescriptor,
    MTLAllocation, MTLClearColor, MTLDevice, MTLDrawable, MTLEvent, MTLLoadAction, MTLPrimitiveType, MTLRenderStages,
    MTLResidencySet, MTLResidencySetDescriptor, MTLScissorRect, MTLSharedEvent, MTLStoreAction, MTLTexture,
    MTLViewport,
};

use crate::render::atlas_kind::AtlasKind;
use crate::render::display_list_encoder::DisplayListEncoder;
use crate::render::draw_batch::BatchSource;
use crate::render::frame_resources::FrameResources;
use crate::render::pixel_rect::PixelRect;
use crate::render::renderer::Renderer;
use crate::render::screen_target::ScreenTarget;
use crate::render::target_geometry::TargetGeometry;

const QUAD_VERTICES: usize = 4;
const TRANSPARENT: MTLClearColor = MTLClearColor {
    red: 0.0,
    green: 0.0,
    blue: 0.0,
    alpha: 0.0,
};
/// A GPU that has not finished a frame of boxes after this long is not going to.
const FRAME_TIMEOUT_MILLISECONDS: u64 = 10_000;

/// Draws display lists into the textures of one surface.
///
/// The GPU reads the buffers of a frame after the frame was handed over, so the renderer
/// rotates through as many sets of buffers as the surface may have frames in flight.
#[derive(Debug)]
pub(crate) struct FrameRenderer {
    renderer: Renderer,
    encoder: DisplayListEncoder,
    frames: Vec<FrameResources>,
    next_frame_resources: usize,
    command_buffer: Retained<ProtocolObject<dyn MTL4CommandBuffer>>,
    render_pass: Retained<MTL4RenderPassDescriptor>,
    /// Holds the buffers of the frames and the textures this renderer was told to keep. Every
    /// command buffer names the sets it uses itself: the list of sets of the queue is shared
    /// by every surface and is not safe to change from two threads.
    residency_set: Retained<ProtocolObject<dyn MTLResidencySet>>,
    /// How many pages of each atlas the residency set already holds.
    resident_mask_pages: usize,
    resident_color_pages: usize,
    /// Takes the number of every frame the GPU finished.
    finished_frames: Retained<ProtocolObject<dyn MTLSharedEvent>>,
    submitted_frames: u64,
}

impl FrameRenderer {
    /// # Panics
    ///
    /// When `frames_in_flight` is zero.
    pub(crate) fn new(renderer: &Renderer, frames_in_flight: usize) -> Self {
        assert!(frames_in_flight > 0, "a surface draws at least one frame at a time");
        let device = renderer.device();
        let residency_set = device
            .newResidencySetWithDescriptor_error(&MTLResidencySetDescriptor::new())
            .unwrap_or_else(|error| panic!("Metal creates a residency set: {}", error.localizedDescription()));
        let frames = (0..frames_in_flight)
            .map(|_| FrameResources::new(device, &residency_set))
            .collect();
        let render_pass = MTL4RenderPassDescriptor::new();
        // SAFETY: a render pass has a color attachment at index 0.
        let attachment = unsafe { render_pass.colorAttachments().objectAtIndexedSubscript(0) };
        attachment.setLoadAction(MTLLoadAction::Clear);
        attachment.setStoreAction(MTLStoreAction::Store);
        attachment.setClearColor(TRANSPARENT);
        Self {
            renderer: renderer.clone(),
            encoder: DisplayListEncoder::default(),
            frames,
            next_frame_resources: 0,
            command_buffer: device.newCommandBuffer().expect("Metal creates a command buffer"),
            render_pass,
            residency_set,
            resident_mask_pages: 0,
            resident_color_pages: 0,
            finished_frames: device.newSharedEvent().expect("Metal creates a shared event"),
            submitted_frames: 0,
        }
    }

    /// Makes a texture this renderer draws into available to its command buffers.
    pub(crate) fn keep_resident(&self, target: &ProtocolObject<dyn MTLTexture>) {
        self.residency_set
            .addAllocation(ProtocolObject::<dyn MTLAllocation>::from_ref(target));
        self.residency_set.commit();
    }

    /// Draws the display list over a cleared `target` and, when the target is the texture of
    /// a drawable, shows it on `screen`. Returns without waiting for the GPU.
    pub(crate) fn render(
        &mut self,
        display_list: &DisplayList,
        geometry: &TargetGeometry,
        target: &ProtocolObject<dyn MTLTexture>,
        screen: Option<ScreenTarget<'_>>,
    ) {
        let position = self.next_frame_resources;
        self.next_frame_resources = (position + 1) % self.frames.len();
        self.wait_for_frame(self.frames[position].last_frame);
        let resources = &mut self.frames[position];
        let mut glyph_atlases = self.renderer.glyph_atlases();
        glyph_atlases.begin_frame();
        self.encoder
            .encode(display_list, geometry, resources, &mut glyph_atlases);
        resources.bind_buffers();
        for (kind, resident_pages) in [
            (AtlasKind::Mask, &mut self.resident_mask_pages),
            (AtlasKind::Color, &mut self.resident_color_pages),
        ] {
            let atlas = glyph_atlases.atlas(kind);
            for page in *resident_pages..atlas.page_count() {
                self.residency_set
                    .addAllocation(ProtocolObject::<dyn MTLAllocation>::from_ref(atlas.texture(page)));
            }
            *resident_pages = atlas.page_count();
        }
        self.residency_set.commit();

        // SAFETY: a render pass has a color attachment at index 0.
        let attachment = unsafe { self.render_pass.colorAttachments().objectAtIndexedSubscript(0) };
        attachment.setTexture(Some(target));
        self.command_buffer
            .beginCommandBufferWithAllocator(&resources.allocator);
        self.command_buffer.useResidencySet(&self.residency_set);
        if let Some(screen) = screen {
            self.command_buffer.useResidencySet(screen.residency_set);
        }
        let pass = self
            .command_buffer
            .renderCommandEncoderWithDescriptor(&self.render_pass)
            .expect("Metal begins a render pass");
        let stages = MTLRenderStages::Vertex | MTLRenderStages::Fragment;
        pass.setArgumentTable_atStages(&resources.argument_table, stages);
        pass.setViewport(MTLViewport {
            originX: 0.0,
            originY: 0.0,
            width: geometry.pixel_width as f64,
            height: geometry.pixel_height as f64,
            znear: 0.0,
            zfar: 1.0,
        });
        for batch in &resources.draws {
            pass.setScissorRect(scissor_rect(batch.scissor));
            match batch.source {
                BatchSource::Rects => pass.setRenderPipelineState(self.renderer.rect_pipeline()),
                BatchSource::Glyphs { atlas, page } => {
                    resources.bind_atlas_page(glyph_atlases.atlas(atlas).texture(page));
                    pass.setArgumentTable_atStages(&resources.argument_table, stages);
                    pass.setRenderPipelineState(self.renderer.glyph_pipeline());
                }
            }
            // SAFETY: the instances of the batch were written to the buffer of their kind that
            // the argument table points at.
            unsafe {
                pass.drawPrimitives_vertexStart_vertexCount_instanceCount_baseInstance(
                    MTLPrimitiveType::TriangleStrip,
                    0,
                    QUAD_VERTICES,
                    batch.instance_count,
                    batch.first_instance,
                );
            }
        }
        pass.endEncoding();
        self.command_buffer.endCommandBuffer();

        let queue = self.renderer.queue();
        if let Some(screen) = screen {
            queue.waitForDrawable(screen.drawable);
        }
        let mut command_buffers = [NonNull::from(&*self.command_buffer)];
        // SAFETY: the pointer is to an array of one live command buffer.
        unsafe { queue.commit_count(NonNull::from(&mut command_buffers).cast(), command_buffers.len()) };
        if let Some(screen) = screen {
            queue.signalDrawable(screen.drawable);
            screen.drawable.present();
        }
        self.submitted_frames += 1;
        queue.signalEvent_value(
            ProtocolObject::<dyn MTLEvent>::from_ref(&*self.finished_frames),
            self.submitted_frames,
        );
        self.frames[position].last_frame = self.submitted_frames;
    }

    /// Blocks until the GPU has drawn every frame handed to it.
    pub(crate) fn wait_until_rendered(&self) {
        self.wait_for_frame(self.submitted_frames);
    }

    /// # Panics
    ///
    /// When the GPU does not finish the frame in time.
    fn wait_for_frame(&self, frame: u64) {
        if self.finished_frames.signaledValue() >= frame {
            return;
        }
        assert!(
            self.finished_frames
                .waitUntilSignaledValue_timeoutMS(frame, FRAME_TIMEOUT_MILLISECONDS),
            "the GPU finishes frame {frame} within {FRAME_TIMEOUT_MILLISECONDS} ms"
        );
    }
}

impl Drop for FrameRenderer {
    /// The GPU may still read the buffers of the last frames.
    fn drop(&mut self) {
        self.wait_until_rendered();
    }
}

fn scissor_rect(pixels: PixelRect) -> MTLScissorRect {
    MTLScissorRect {
        x: pixels.x,
        y: pixels.y,
        width: pixels.width,
        height: pixels.height,
    }
}
