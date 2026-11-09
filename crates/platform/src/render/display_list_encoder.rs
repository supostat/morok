use bytemuck::Zeroable;
use morok_core::{BoxFrame, DisplayItem, DisplayList, GradientFill, GradientShape, Rect, Sides};
use morok_style::{Color, ShadowLayer};

use crate::render::active_clip::ActiveClip;
use crate::render::draw_batch::DrawBatch;
use crate::render::frame_resources::FrameResources;
use crate::render::frame_uniforms::FrameUniforms;
use crate::render::gradient_stop_entry::GradientStopEntry;
use crate::render::pixel_rect::PixelRect;
use crate::render::premultiplied_color::premultiplied;
use crate::render::rect_instance::RectInstance;
use crate::render::rect_kind::RectKind;
use crate::render::rounded_rect::RoundedRect;
use crate::render::target_geometry::TargetGeometry;

/// The standard deviation of the Gaussian of a CSS blur radius.
const DEVIATION_PER_BLUR_RADIUS: f32 = 0.5;
/// A Gaussian has faded out three standard deviations away.
const BLUR_REACH_PER_DEVIATION: f32 = 3.0;
/// An anti-aliased edge still covers a part of the pixels whose centers lie this far outside it.
const EDGE_REACH_IN_PIXELS: f32 = 0.5;

/// Turns a display list into the instances, gradient stops, clips and draw calls of a frame,
/// written straight into the buffers the GPU reads.
#[derive(Debug, Default)]
pub(crate) struct DisplayListEncoder {
    /// The clips enclosing the item being encoded, innermost last.
    active_clips: Vec<ActiveClip>,
}

impl DisplayListEncoder {
    pub(crate) fn encode(
        &mut self,
        display_list: &DisplayList,
        geometry: &TargetGeometry,
        resources: &mut FrameResources,
    ) {
        resources.begin_frame(FrameUniforms {
            viewport_size: [geometry.viewport.width, geometry.viewport.height],
            scale: geometry.scale,
            padding: 0.0,
        });
        self.active_clips.clear();
        self.active_clips.push(ActiveClip {
            scissor: geometry.pixels(),
            rounded_start: 0,
            rounded_count: 0,
        });
        let mut frame = FrameEncoding {
            display_list,
            scale: geometry.scale,
            active_clips: &mut self.active_clips,
            resources,
        };
        for item in display_list.items() {
            frame.encode_item(item);
        }
    }
}

/// One run of the encoder over a display list.
struct FrameEncoding<'a> {
    display_list: &'a DisplayList,
    scale: f32,
    active_clips: &'a mut Vec<ActiveClip>,
    resources: &'a mut FrameResources,
}

impl FrameEncoding<'_> {
    fn encode_item(&mut self, item: &DisplayItem) {
        match *item {
            DisplayItem::Shadow { frame, layer } if layer.inset => self.inset_shadow(&self.snapped(&frame), &layer),
            DisplayItem::Shadow { frame, layer } => self.shadow(&self.snapped(&frame), &layer),
            DisplayItem::Background { frame, color } => self.background(&self.snapped(&frame), color),
            DisplayItem::Gradient { frame, gradient } => self.gradient(&self.snapped(&frame), gradient),
            DisplayItem::Border { frame, colors } => self.border(&self.snapped(&frame), colors),
            DisplayItem::Outline {
                frame,
                width,
                offset,
                color,
            } => self.outline(&self.snapped(&frame), width, offset, color),
            DisplayItem::PushClip { frame } => self.push_clip(&self.snapped(&frame)),
            DisplayItem::PopClip => {
                self.active_clips.pop();
            }
        }
    }

    /// The frame with every edge of its border box on the nearest edge between device pixels.
    /// Browsers paint boxes this way: a box laid out between pixels keeps sharp sides instead
    /// of blurring them over two pixels.
    fn snapped(&self, frame: &BoxFrame) -> BoxFrame {
        let snap = |edge: f32| (edge * self.scale).round() / self.scale;
        let left = snap(frame.border_box.x);
        let top = snap(frame.border_box.y);
        BoxFrame {
            border_box: Rect::new(
                left,
                top,
                snap(frame.border_box.right()) - left,
                snap(frame.border_box.bottom()) - top,
            ),
            ..*frame
        }
    }

    fn active_clip(&self) -> ActiveClip {
        *self
            .active_clips
            .last()
            .expect("the clip of the whole target is never popped")
    }

    fn background(&mut self, frame: &BoxFrame, color: Color) {
        let shape = RoundedRect::border_box(frame);
        self.emit(RectInstance {
            bounds: self.bounds_of(&shape.rect()),
            outer: shape,
            colors: [premultiplied(color); 4],
            kind: RectKind::Fill as u32,
            ..RectInstance::zeroed()
        });
    }

    /// One tile of the gradient covers the padding box; a padding box without area has no tile
    /// to repeat, and CSS paints nothing then.
    fn gradient(&mut self, frame: &BoxFrame, gradient: GradientFill) {
        let tile = frame.padding_box();
        let shape = RoundedRect::border_box(frame);
        let bounds = self.bounds_of(&shape.rect());
        if tile.is_empty() || !self.is_visible(bounds) {
            return;
        }
        let (kind, parameters) = match gradient.shape {
            GradientShape::Linear { angle } => {
                let (sine, cosine) = angle.to_radians().sin_cos();
                let line_length = (tile.width * sine).abs() + (tile.height * cosine).abs();
                (RectKind::LinearGradient, [sine, -cosine, line_length, 0.0])
            }
            GradientShape::Radial {
                center,
                horizontal_radius,
                vertical_radius,
            } => (
                RectKind::RadialGradient,
                [center.x, center.y, horizontal_radius, vertical_radius],
            ),
        };
        let stops_start = self.resources.gradient_stops.len();
        for stop in self.display_list.gradient_stops(gradient.stops) {
            self.resources.gradient_stops.push(GradientStopEntry {
                color: premultiplied(stop.color),
                offset: stop.offset,
                padding: [0.0; 3],
            });
        }
        self.emit(RectInstance {
            bounds,
            outer: shape,
            inner: RoundedRect {
                rect: [tile.x, tile.y, tile.width, tile.height],
                ..RoundedRect::zeroed()
            },
            parameters,
            kind: kind as u32,
            gradient_stops_start: stops_start as u32,
            gradient_stops_count: (self.resources.gradient_stops.len() - stops_start) as u32,
            ..RectInstance::zeroed()
        });
    }

    fn border(&mut self, frame: &BoxFrame, colors: Sides<Color>) {
        let outer = RoundedRect::border_box(frame);
        self.emit(RectInstance {
            bounds: self.bounds_of(&outer.rect()),
            outer,
            inner: RoundedRect::padding_box(frame),
            colors: [colors.top, colors.right, colors.bottom, colors.left].map(premultiplied),
            kind: RectKind::Border as u32,
            ..RectInstance::zeroed()
        });
    }

    /// An outline is a border drawn outside the box: a ring between the border box moved out
    /// by the offset and the same box moved out by the offset and the width.
    fn outline(&mut self, frame: &BoxFrame, width: f32, offset: f32, color: Color) {
        let border_box = RoundedRect::border_box(frame);
        let outer = border_box.outset(offset + width);
        self.emit(RectInstance {
            bounds: self.bounds_of(&outer.rect()),
            outer,
            inner: border_box.outset(offset),
            colors: [premultiplied(color); 4],
            kind: RectKind::Border as u32,
            ..RectInstance::zeroed()
        });
    }

    /// The shadow is the border box grown by the spread, moved by the offset and blurred; the
    /// box itself hides the part under it.
    fn shadow(&mut self, frame: &BoxFrame, layer: &ShadowLayer) {
        let border_box = RoundedRect::border_box(frame);
        let caster = border_box
            .outset(layer.spread.0)
            .translated(layer.offset_x.0, layer.offset_y.0);
        let deviation = layer.blur.0 * DEVIATION_PER_BLUR_RADIUS;
        let reach = deviation * BLUR_REACH_PER_DEVIATION;
        self.emit(RectInstance {
            bounds: self.bounds_of(&caster.rect().expanded(Sides::uniform(reach))),
            outer: caster,
            inner: border_box,
            colors: [premultiplied(layer.color); 4],
            parameters: [deviation, 0.0, 0.0, 0.0],
            kind: RectKind::Shadow as u32,
            ..RectInstance::zeroed()
        });
    }

    /// An inset shadow is cast by everything around an opening: the padding box shrunk by the
    /// spread and moved by the offset. It shows only inside the padding box.
    fn inset_shadow(&mut self, frame: &BoxFrame, layer: &ShadowLayer) {
        let padding_box = RoundedRect::padding_box(frame);
        self.emit(RectInstance {
            bounds: self.bounds_of(&padding_box.rect()),
            outer: padding_box
                .outset(-layer.spread.0)
                .translated(layer.offset_x.0, layer.offset_y.0),
            inner: padding_box,
            colors: [premultiplied(layer.color); 4],
            parameters: [layer.blur.0 * DEVIATION_PER_BLUR_RADIUS, 0.0, 0.0, 0.0],
            kind: RectKind::InsetShadow as u32,
            ..RectInstance::zeroed()
        });
    }

    /// A rectangular clip only narrows the scissor. A rounded one also joins the masks of the
    /// items inside it; the masks of an item are one run of the clip buffer, so the run of the
    /// enclosing clips is repeated in front of the new shape.
    fn push_clip(&mut self, frame: &BoxFrame) {
        let enclosing = self.active_clip();
        let shape = RoundedRect::padding_box(frame);
        let clip = if shape.is_rounded() {
            let rounded_start = self.resources.clips.len();
            for position in 0..enclosing.rounded_count {
                let enclosing_shape = self.resources.clips.get(enclosing.rounded_start + position);
                self.resources.clips.push(enclosing_shape);
            }
            self.resources.clips.push(shape);
            ActiveClip {
                scissor: enclosing
                    .scissor
                    .intersection(&PixelRect::covering(&shape.rect(), self.scale)),
                rounded_start,
                rounded_count: enclosing.rounded_count + 1,
            }
        } else {
            ActiveClip {
                scissor: enclosing
                    .scissor
                    .intersection(&PixelRect::nearest(&shape.rect(), self.scale)),
                ..enclosing
            }
        };
        self.active_clips.push(clip);
    }

    /// The quad of an instance that draws inside `drawn`: origin and size.
    fn bounds_of(&self, drawn: &Rect) -> [f32; 4] {
        let quad = drawn.expanded(Sides::uniform(EDGE_REACH_IN_PIXELS / self.scale));
        [quad.x, quad.y, quad.width, quad.height]
    }

    fn is_visible(&self, bounds: [f32; 4]) -> bool {
        let [x, y, width, height] = bounds;
        let pixels = PixelRect::covering(&Rect::new(x, y, width, height), self.scale);
        !pixels.intersection(&self.active_clip().scissor).is_empty()
    }

    /// Appends the instance under the clip in force, unless the clip hides all of it.
    fn emit(&mut self, mut instance: RectInstance) {
        if !self.is_visible(instance.bounds) {
            return;
        }
        let clip = self.active_clip();
        instance.clips_start = clip.rounded_start as u32;
        instance.clips_count = clip.rounded_count as u32;
        let position = self.resources.instances.len();
        self.resources.instances.push(instance);
        match self.resources.draws.last_mut() {
            Some(batch) if batch.scissor == clip.scissor => batch.instance_count += 1,
            _ => self.resources.draws.push(DrawBatch {
                scissor: clip.scissor,
                first_instance: position,
                instance_count: 1,
            }),
        }
    }
}
