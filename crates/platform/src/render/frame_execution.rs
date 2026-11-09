use objc2::Message;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTL4CommandBuffer, MTL4CommandEncoder, MTL4RenderCommandEncoder, MTL4RenderPassDescriptor, MTL4VisibilityOptions,
    MTLLoadAction, MTLPrimitiveType, MTLRenderStages, MTLResidencySet, MTLScissorRect, MTLStages, MTLTexture,
    MTLViewport,
};

use crate::render::blur_uniforms::BlurUniforms;
use crate::render::clip_shape::ClipShape;
use crate::render::composite_instance::CompositeInstance;
use crate::render::draw_batch::{BatchSource, DrawBatch};
use crate::render::frame_command::FrameCommand;
use crate::render::frame_resources::FrameResources;
use crate::render::frame_uniforms::FrameUniforms;
use crate::render::glyph_atlases::GlyphAtlases;
use crate::render::group_pass::BackdropPass;
use crate::render::pixel_rect::PixelRect;
use crate::render::renderer::Renderer;
use crate::render::target_geometry::TargetGeometry;

const QUAD_VERTICES: usize = 4;
/// A Gaussian has faded out three standard deviations away.
const BLUR_REACH_PER_DEVIATION: f32 = 3.0;
const HORIZONTAL: [f32; 2] = [1.0, 0.0];
const VERTICAL: [f32; 2] = [0.0, 1.0];

/// What a pass draws into: a texture showing a window of the surface.
#[derive(Debug)]
pub(crate) struct Target {
    texture: Retained<ProtocolObject<dyn MTLTexture>>,
    /// The device pixels of the surface the target shows; the texture may be larger.
    pixels: PixelRect,
    /// The entry of the uniforms of this target.
    uniforms: usize,
}

/// A group whose end has not been drawn yet.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OpenGroup {
    /// The texture of the pool the picture of the group is drawn into; none for a group that
    /// covers no pixel, whose commands draw nothing.
    picture: Option<usize>,
}

/// Runs the commands of one encoded frame on the GPU: the draws of the surface, and for every
/// group a picture of its own that is blurred and laid over the target the group began on.
pub(crate) struct FrameExecution<'a> {
    pub(crate) renderer: &'a Renderer,
    pub(crate) resources: &'a mut FrameResources,
    pub(crate) glyph_atlases: &'a GlyphAtlases,
    pub(crate) command_buffer: &'a ProtocolObject<dyn MTL4CommandBuffer>,
    pub(crate) render_pass: &'a MTL4RenderPassDescriptor,
    pub(crate) residency_set: &'a ProtocolObject<dyn MTLResidencySet>,
    pub(crate) geometry: &'a TargetGeometry,
    /// Scratch space: the targets being drawn into, the surface first and the pictures of the
    /// open groups after it, and the open groups themselves.
    pub(crate) targets: &'a mut Vec<Target>,
    pub(crate) open_groups: &'a mut Vec<OpenGroup>,
    /// The pass that is open; none between passes.
    pub(crate) pass: Option<Retained<ProtocolObject<dyn MTL4RenderCommandEncoder>>>,
}

impl FrameExecution<'_> {
    /// Draws every command into `surface`, which is cleared first.
    pub(crate) fn run(mut self, surface: &ProtocolObject<dyn MTLTexture>) {
        self.targets.clear();
        self.open_groups.clear();
        self.targets.push(Target {
            texture: surface.retain(),
            pixels: self.geometry.pixels(),
            uniforms: 0,
        });
        self.begin_pass(MTLLoadAction::Clear);
        for position in 0..self.resources.commands.len() {
            match self.resources.commands[position] {
                FrameCommand::Draw(batch) => self.draw(&batch),
                FrameCommand::BeginGroup(group) => self.begin_group(group),
                FrameCommand::EndGroup(group) => self.end_group(group),
            }
        }
        self.end_pass();
    }

    fn target(&self) -> &Target {
        self.targets.last().expect("a frame draws into a target")
    }

    fn pass(&self) -> &ProtocolObject<dyn MTL4RenderCommandEncoder> {
        self.pass.as_deref().expect("a pass is open while commands are drawn")
    }

    /// Begins a pass on the current target, which keeps what it holds or is cleared.
    fn begin_pass(&mut self, load_action: MTLLoadAction) {
        let target = self.target();
        // SAFETY: a render pass has a color attachment at index 0.
        let attachment = unsafe { self.render_pass.colorAttachments().objectAtIndexedSubscript(0) };
        attachment.setTexture(Some(&target.texture));
        attachment.setLoadAction(load_action);
        let pass = self
            .command_buffer
            .renderCommandEncoderWithDescriptor(self.render_pass)
            .expect("Metal begins a render pass");
        // Metal 4 tracks no hazards between passes: a pass that reads what an earlier pass
        // drew waits for it here.
        pass.barrierAfterQueueStages_beforeStages_visibilityOptions(
            MTLStages::Fragment,
            MTLStages::Fragment,
            MTL4VisibilityOptions::Device,
        );
        pass.setViewport(MTLViewport {
            originX: 0.0,
            originY: 0.0,
            width: target.pixels.width as f64,
            height: target.pixels.height as f64,
            znear: 0.0,
            zfar: 1.0,
        });
        self.resources.bind_uniforms(target.uniforms);
        self.bind_argument_table(&pass);
        self.pass = Some(pass);
    }

    fn end_pass(&mut self) {
        if let Some(pass) = self.pass.take() {
            pass.endEncoding();
        }
    }

    fn bind_argument_table(&self, pass: &ProtocolObject<dyn MTL4RenderCommandEncoder>) {
        pass.setArgumentTable_atStages(
            &self.resources.argument_table,
            MTLRenderStages::Vertex | MTLRenderStages::Fragment,
        );
    }

    /// The scissor of a batch in the pixels of the current target.
    fn scissor_in_target(&self, scissor: PixelRect) -> Option<MTLScissorRect> {
        let target = self.target().pixels;
        let clipped = scissor.intersection(&target);
        (!clipped.is_empty()).then_some(MTLScissorRect {
            x: clipped.x - target.x,
            y: clipped.y - target.y,
            width: clipped.width,
            height: clipped.height,
        })
    }

    fn draw(&mut self, batch: &DrawBatch) {
        let Some(scissor) = self.scissor_in_target(batch.scissor) else {
            return;
        };
        let pass = self.pass();
        pass.setScissorRect(scissor);
        match batch.source {
            BatchSource::Rects => pass.setRenderPipelineState(self.renderer.rect_pipeline()),
            BatchSource::Glyphs { atlas, page } => {
                self.resources
                    .bind_texture(self.glyph_atlases.atlas(atlas).texture(page));
                self.bind_argument_table(pass);
                pass.setRenderPipelineState(self.renderer.glyph_pipeline());
            }
        }
        draw_quads(pass, batch.instance_count, batch.first_instance);
    }

    /// Opens a group: the backdrop of its element is filtered in place, and a picture is
    /// begun for everything inside the group.
    fn begin_group(&mut self, group: usize) {
        let pass = self.resources.groups[group];
        self.end_pass();
        if let Some(backdrop) = pass.backdrop {
            self.filter_backdrop(&backdrop);
        }
        if pass.pixels.is_empty() {
            self.begin_pass(MTLLoadAction::Load);
            self.open_groups.push(OpenGroup { picture: None });
            return;
        }
        let picture = self.acquire(pass.pixels);
        let uniforms = self.resources.uniforms.len();
        self.resources.uniforms.push(FrameUniforms {
            viewport_size: [self.geometry.viewport.width, self.geometry.viewport.height],
            scale: self.geometry.scale,
            padding: 0.0,
            origin: [pass.pixels.x as f32, pass.pixels.y as f32],
            target_size: [pass.pixels.width as f32, pass.pixels.height as f32],
        });
        self.targets.push(Target {
            texture: self.resources.textures.texture(picture).retain(),
            pixels: pass.pixels,
            uniforms,
        });
        self.open_groups.push(OpenGroup { picture: Some(picture) });
        self.begin_pass(MTLLoadAction::Clear);
    }

    /// Closes a group: its picture, blurred when the group asks for it, is laid over the
    /// target the group began on.
    fn end_group(&mut self, group: usize) {
        let open = self.open_groups.pop().expect("a group is open when it ends");
        let Some(mut picture) = open.picture else {
            return;
        };
        let pass = self.resources.groups[group];
        self.end_pass();
        self.targets.pop();
        if let Some(deviation) = pass.blur {
            picture = self.blur(picture, pass.pixels, deviation);
        }
        self.begin_pass(MTLLoadAction::Load);
        self.composite(
            picture,
            CompositeInstance {
                pixels: self.quad_of(pass.pixels),
                opacity: pass.opacity,
                blend_mode: pass.blend_mode,
                filters_start: pass.filters_start,
                filters_count: pass.filters_count,
                clip: ClipShape::default(),
                has_clip: 0,
                padding: [0; 3],
            },
        );
    }

    /// Copies what the target holds under the frame of the element into a picture, filters it
    /// and lays it back inside the frame. The pass on the target is not open meanwhile.
    fn filter_backdrop(&mut self, backdrop: &BackdropPass) {
        if backdrop.pixels.is_empty() {
            return;
        }
        let source = self.target().texture.clone();
        let source_pixels = self.target().pixels;
        let deviation = backdrop.blur.unwrap_or(0.0);
        let first = self.acquire(backdrop.pixels);
        self.blur_pass(
            &source,
            source_pixels,
            first,
            backdrop.pixels,
            BlurPass::horizontal(deviation),
        );
        let second = self.acquire(backdrop.pixels);
        let first_texture = self.resources.textures.texture(first).retain();
        self.blur_pass(
            &first_texture,
            backdrop.pixels,
            second,
            backdrop.pixels,
            BlurPass::vertical(deviation),
        );
        self.begin_pass(MTLLoadAction::Load);
        self.composite(
            second,
            CompositeInstance {
                pixels: self.quad_of(backdrop.pixels),
                opacity: 1.0,
                blend_mode: 0,
                filters_start: backdrop.filters_start,
                filters_count: backdrop.filters_count,
                clip: backdrop.clip,
                has_clip: 1,
                padding: [0; 3],
            },
        );
        self.end_pass();
    }

    /// Blurs the picture in two passes and returns the texture holding the result.
    fn blur(&mut self, picture: usize, pixels: PixelRect, deviation: f32) -> usize {
        let first = self.acquire(pixels);
        let picture_texture = self.resources.textures.texture(picture).retain();
        self.blur_pass(&picture_texture, pixels, first, pixels, BlurPass::horizontal(deviation));
        let second = self.acquire(pixels);
        let first_texture = self.resources.textures.texture(first).retain();
        self.blur_pass(&first_texture, pixels, second, pixels, BlurPass::vertical(deviation));
        second
    }

    /// One pass of the blur from `source`, which shows `source_pixels` of the surface, into the
    /// pool texture `destination`, which shows `destination_pixels`.
    fn blur_pass(
        &mut self,
        source: &ProtocolObject<dyn MTLTexture>,
        source_pixels: PixelRect,
        destination: usize,
        destination_pixels: PixelRect,
        blur: BlurPass,
    ) {
        let radius = if blur.deviation > 0.0 {
            (blur.deviation * BLUR_REACH_PER_DEVIATION).ceil() as u32
        } else {
            0
        };
        let entry = self.resources.blur_passes.len();
        self.resources.blur_passes.push(BlurUniforms {
            deviation: blur.deviation,
            radius,
            direction: blur.direction,
            source_offset: [
                destination_pixels.x as f32 - source_pixels.x as f32,
                destination_pixels.y as f32 - source_pixels.y as f32,
            ],
            source_size: [source_pixels.width as f32, source_pixels.height as f32],
        });
        self.targets.push(Target {
            texture: self.resources.textures.texture(destination).retain(),
            pixels: destination_pixels,
            uniforms: self.target().uniforms,
        });
        self.begin_pass(MTLLoadAction::Clear);
        self.resources.bind_blur_pass(entry);
        self.resources.bind_texture(source);
        let pass = self.pass();
        self.bind_argument_table(pass);
        pass.setScissorRect(MTLScissorRect {
            x: 0,
            y: 0,
            width: destination_pixels.width,
            height: destination_pixels.height,
        });
        pass.setRenderPipelineState(self.renderer.blur_pipeline());
        draw_quads(pass, 1, 0);
        self.end_pass();
        self.targets.pop();
    }

    /// Lays the pool texture `picture` over the current target as the instance says.
    fn composite(&mut self, picture: usize, instance: CompositeInstance) {
        let position = self.resources.composites.len();
        self.resources.composites.push(instance);
        self.resources.bind_composites();
        let texture = self.resources.textures.texture(picture).retain();
        self.resources.bind_texture(&texture);
        let pass = self.pass();
        self.bind_argument_table(pass);
        let target = self.target().pixels;
        pass.setScissorRect(MTLScissorRect {
            x: 0,
            y: 0,
            width: target.width,
            height: target.height,
        });
        pass.setRenderPipelineState(self.renderer.composite_pipeline());
        draw_quads(pass, 1, position);
    }

    /// The quad of `pixels` of the surface in the pixels of the current target.
    fn quad_of(&self, pixels: PixelRect) -> [f32; 4] {
        let target = self.target().pixels;
        [
            pixels.x as f32 - target.x as f32,
            pixels.y as f32 - target.y as f32,
            pixels.width as f32,
            pixels.height as f32,
        ]
    }

    /// Takes a texture of the pool large enough for `pixels` of the surface.
    fn acquire(&mut self, pixels: PixelRect) -> usize {
        self.resources
            .textures
            .acquire(self.renderer.device(), self.residency_set, pixels.width, pixels.height)
    }
}

/// One direction of a separable blur.
#[derive(Clone, Copy, Debug)]
struct BlurPass {
    deviation: f32,
    direction: [f32; 2],
}

impl BlurPass {
    fn horizontal(deviation: f32) -> Self {
        Self {
            deviation,
            direction: HORIZONTAL,
        }
    }

    fn vertical(deviation: f32) -> Self {
        Self {
            deviation,
            direction: VERTICAL,
        }
    }
}

fn draw_quads(pass: &ProtocolObject<dyn MTL4RenderCommandEncoder>, instance_count: usize, first_instance: usize) {
    // SAFETY: the instances were written to the buffer of their kind that the argument table
    // points at.
    unsafe {
        pass.drawPrimitives_vertexStart_vertexCount_instanceCount_baseInstance(
            MTLPrimitiveType::TriangleStrip,
            0,
            QUAD_VERTICES,
            instance_count,
            first_instance,
        );
    }
}
