use std::fmt::{Debug, Formatter, Result};

use bytemuck::Zeroable;
use morok_core::Rect;
use morok_style::Color;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTL4RenderCommandEncoder, MTLPrimitiveType, MTLRenderStages};

use crate::canvas::canvas_region::CanvasRegion;
use crate::render::frame_resources::FrameResources;
use crate::render::premultiplied_color::premultiplied;
use crate::render::rect_instance::RectInstance;
use crate::render::rect_kind::RectKind;
use crate::render::renderer::Renderer;
use crate::render::rounded_rect::RoundedRect;

const QUAD_VERTICES: usize = 4;

/// The pass of a frame handed to the application for one canvas. The kit has clipped the pass
/// to the canvas; what is drawn through the Metal encoder directly must leave the pipeline
/// state as it found it, or go through the fills the encoder offers.
pub struct CanvasEncoder<'a> {
    pub(crate) pass: &'a ProtocolObject<dyn MTL4RenderCommandEncoder>,
    pub(crate) region: CanvasRegion,
    pub(crate) resources: &'a mut FrameResources,
    pub(crate) renderer: &'a Renderer,
    /// The transform of the canvas, in the transform buffer of the frame.
    pub(crate) transform: u32,
}

impl CanvasEncoder<'_> {
    /// The content box of the canvas on the surface.
    pub fn region(&self) -> CanvasRegion {
        self.region
    }

    /// The Metal 4 encoder of the pass, for drawing with pipelines of the application's own.
    pub fn metal_encoder(&self) -> &ProtocolObject<dyn MTL4RenderCommandEncoder> {
        self.pass
    }

    /// Fills a rectangle given in CSS pixels from the top left corner of the canvas.
    pub fn fill(&mut self, rect: Rect, color: Color) {
        let on_surface = Rect::new(self.region.x + rect.x, self.region.y + rect.y, rect.width, rect.height);
        let position = self.resources.instances.len();
        self.resources.instances.push(RectInstance {
            bounds: [on_surface.x, on_surface.y, on_surface.width, on_surface.height],
            outer: RoundedRect {
                rect: [on_surface.x, on_surface.y, on_surface.width, on_surface.height],
                ..RoundedRect::zeroed()
            },
            colors: [premultiplied(color); 4],
            kind: RectKind::Fill as u32,
            transform: self.transform,
            ..RectInstance::zeroed()
        });
        self.resources.bind_instances();
        self.pass.setArgumentTable_atStages(
            &self.resources.argument_table,
            MTLRenderStages::Vertex | MTLRenderStages::Fragment,
        );
        self.pass.setRenderPipelineState(self.renderer.rect_pipeline());
        // SAFETY: the instance was written to the buffer the argument table points at.
        unsafe {
            self.pass
                .drawPrimitives_vertexStart_vertexCount_instanceCount_baseInstance(
                    MTLPrimitiveType::TriangleStrip,
                    0,
                    QUAD_VERTICES,
                    1,
                    position,
                );
        }
    }
}

impl Debug for CanvasEncoder<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("CanvasEncoder")
            .field("region", &self.region)
            .finish_non_exhaustive()
    }
}
