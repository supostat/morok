use bytemuck::Zeroable;
use morok_core::{
    BoxFrame, CanvasId, DisplayItem, DisplayList, Filter, FontId, GradientFill, GradientShape, GroupEffects, IconId,
    ImageId, ItemRange, Point, Rect, Sides, Transform,
};
use morok_style::{BlendMode, Color, ShadowLayer};

use crate::media::icon_library::IconLibrary;
use crate::media::image_store::{ImageState, ImageStore};
use crate::render::active_clip::ActiveClip;
use crate::render::atlas_kind::AtlasKind;
use crate::render::atlases::Atlases;
use crate::render::clip_shape::ClipShape;
use crate::render::draw_batch::PictureSource;
use crate::render::draw_batcher::DrawBatcher;
use crate::render::filter_entry::FilterEntry;
use crate::render::frame_command::FrameCommand;
use crate::render::frame_resources::FrameResources;
use crate::render::frame_uniforms::FrameUniforms;
use crate::render::glyph_instance::GlyphInstance;
use crate::render::glyph_key::GlyphKey;
use crate::render::glyph_rasterizer::SUBPIXEL_PHASES;
use crate::render::gradient_stop_entry::GradientStopEntry;
use crate::render::group_pass::{BackdropPass, GroupPass};
use crate::render::image_instance::ImageInstance;
use crate::render::pixel_rect::PixelRect;
use crate::render::premultiplied_color::premultiplied;
use crate::render::rect_instance::RectInstance;
use crate::render::rect_kind::RectKind;
use crate::render::rounded_rect::RoundedRect;
use crate::render::target_geometry::TargetGeometry;
use crate::render::transform_entry::TransformEntry;

/// The standard deviation of the Gaussian of a CSS blur radius.
const DEVIATION_PER_BLUR_RADIUS: f32 = 0.5;
/// A Gaussian has faded out three standard deviations away.
const BLUR_REACH_PER_DEVIATION: f32 = 3.0;
/// An anti-aliased edge still covers a part of the pixels whose centers lie this far outside it.
const EDGE_REACH_IN_PIXELS: f32 = 0.5;
/// The transform every instance outside a transformed group uses.
const IDENTITY_TRANSFORM: u32 = 0;

/// Turns a display list into the instances, gradient stops, clips, transforms and commands of
/// a frame, written straight into the buffers the GPU reads. Glyphs are taken from the
/// atlases, and rasterized into them when a frame first draws them.
#[derive(Debug, Default)]
pub(crate) struct DisplayListEncoder {
    /// The clips enclosing the item being encoded, innermost last.
    active_clips: Vec<ActiveClip>,
    /// The groups enclosing the item being encoded, innermost last.
    open_groups: Vec<OpenGroup>,
    batcher: DrawBatcher,
}

/// A group whose pop has not been encoded yet.
#[derive(Clone, Copy, Debug)]
struct OpenGroup {
    /// The transform of the group on the surface.
    transform: Transform,
    /// The same transform in the transform buffer of the frame.
    transform_entry: u32,
    /// The pass of the group, when it draws its picture apart.
    pass: Option<usize>,
}

impl DisplayListEncoder {
    pub(crate) fn encode(
        &mut self,
        display_list: &DisplayList,
        geometry: &TargetGeometry,
        resources: &mut FrameResources,
        glyph_atlases: &mut Atlases,
        media: MediaSources<'_>,
    ) {
        resources.begin_frame();
        resources.uniforms.push(FrameUniforms {
            viewport_size: [geometry.viewport.width, geometry.viewport.height],
            scale: geometry.scale,
            padding: 0.0,
            origin: [0.0, 0.0],
            target_size: [geometry.pixel_width as f32, geometry.pixel_height as f32],
        });
        self.active_clips.clear();
        self.active_clips.push(ActiveClip {
            scissor: geometry.pixels(),
            rounded_start: 0,
            rounded_count: 0,
        });
        self.open_groups.clear();
        self.batcher.begin_frame(geometry.pixels());
        let mut frame = FrameEncoding {
            display_list,
            scale: geometry.scale,
            target: geometry.pixels(),
            active_clips: &mut self.active_clips,
            open_groups: &mut self.open_groups,
            batcher: &mut self.batcher,
            resources,
            glyph_atlases,
            media,
        };
        let mut items = display_list.items().iter();
        while let Some(item) = items.next() {
            if let Some(boxed) = BoxedItems::of(item, items.as_slice().first()) {
                frame.encode_box(&boxed);
                items.next();
            } else {
                frame.encode_item(item);
            }
        }
        self.batcher.finish(&mut resources.commands);
    }
}

/// The icons and images a frame may draw.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MediaSources<'a> {
    pub(crate) icons: &'a IconLibrary,
    pub(crate) images: &'a ImageStore,
}

/// One run of the encoder over a display list.
struct FrameEncoding<'a> {
    display_list: &'a DisplayList,
    scale: f32,
    /// The pixels of the whole target.
    target: PixelRect,
    active_clips: &'a mut Vec<ActiveClip>,
    open_groups: &'a mut Vec<OpenGroup>,
    batcher: &'a mut DrawBatcher,
    resources: &'a mut FrameResources,
    glyph_atlases: &'a mut Atlases,
    media: MediaSources<'a>,
}

/// A background and the border drawn right after it around the same box.
struct BoxedItems {
    frame: BoxFrame,
    fill: Color,
    border_colors: Sides<Color>,
}

impl BoxedItems {
    /// The background and the border of one box when the two items are those, in that order:
    /// they are drawn as one instance.
    fn of(item: &DisplayItem, next: Option<&DisplayItem>) -> Option<Self> {
        let DisplayItem::Background { frame, color } = *item else {
            return None;
        };
        let Some(DisplayItem::Border {
            frame: border_frame,
            colors,
        }) = next
        else {
            return None;
        };
        (*border_frame == frame).then_some(Self {
            frame,
            fill: color,
            border_colors: *colors,
        })
    }
}

impl FrameEncoding<'_> {
    fn encode_box(&mut self, boxed: &BoxedItems) {
        let frame = self.snapped(&boxed.frame);
        let outer = RoundedRect::border_box(&frame);
        let [red, green, blue, alpha] = premultiplied(boxed.fill);
        self.emit(RectInstance {
            bounds: self.bounds_of(&outer.rect()),
            outer,
            inner: RoundedRect::padding_box(&frame),
            colors: [
                boxed.border_colors.top,
                boxed.border_colors.right,
                boxed.border_colors.bottom,
                boxed.border_colors.left,
            ]
            .map(premultiplied),
            parameters: [red, green, blue, alpha],
            kind: RectKind::Box as u32,
            ..RectInstance::zeroed()
        });
    }

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
            DisplayItem::GlyphRun {
                origin,
                font,
                glyphs,
                color,
            } => self.glyph_run(origin, font, glyphs, color),
            DisplayItem::TextDecoration { rect, color } => self.text_decoration(&rect, color),
            DisplayItem::Fill { rect, color } => self.fill(&rect, color),
            DisplayItem::PushClip { frame } => self.push_clip(&self.snapped(&frame)),
            DisplayItem::PopClip => {
                self.active_clips.pop();
            }
            DisplayItem::PushGroup { effects } => self.push_group(&effects),
            DisplayItem::PopGroup => self.pop_group(),
            DisplayItem::Image { image, dest, clip } => self.image(image, &dest, &clip),
            DisplayItem::Icon { icon, dest, color } => self.icon(icon, &dest, color),
            DisplayItem::Canvas { canvas, dest } => self.canvas(canvas, &dest),
        }
    }

    /// The part of the picture inside `clip`, stretched as `dest` stretches the whole picture.
    /// A small picture comes from the color atlas, a large one from a texture of its own; one
    /// not decoded yet draws nothing.
    fn image(&mut self, image: ImageId, dest: &Rect, clip: &Rect) {
        let Some(ImageState::Decoded(decoded)) = self.media.images.state(image) else {
            return;
        };
        let visible = Rect::new(
            dest.x.max(clip.x),
            dest.y.max(clip.y),
            dest.right().min(clip.right()) - dest.x.max(clip.x),
            dest.bottom().min(clip.bottom()) - dest.y.max(clip.y),
        );
        if visible.is_empty() || dest.is_empty() {
            return;
        }
        let bounds = [visible.x, visible.y, visible.width, visible.height];
        let pixels = self.visible_pixels(bounds);
        if pixels.is_empty() {
            return;
        }
        let texels_per_pixel_x = decoded.width as f32 / dest.width;
        let texels_per_pixel_y = decoded.height as f32 / dest.height;
        let (source, origin) = match self.glyph_atlases.image(image, &decoded) {
            Some(placed) => (
                PictureSource::AtlasPage {
                    atlas: placed.atlas,
                    page: placed.entry.page,
                },
                [placed.entry.x as f32, placed.entry.y as f32],
            ),
            None => (PictureSource::ImageTexture(image), [0.0, 0.0]),
        };
        let clip_shape = self.active_clip();
        let position = self.resources.image_instances.len();
        self.resources.image_instances.push(ImageInstance {
            bounds,
            texels: [
                origin[0] + (visible.x - dest.x) * texels_per_pixel_x,
                origin[1] + (visible.y - dest.y) * texels_per_pixel_y,
                visible.width * texels_per_pixel_x,
                visible.height * texels_per_pixel_y,
            ],
            clips_start: clip_shape.rounded_start as u32,
            clips_count: clip_shape.rounded_count as u32,
            transform: self.transform_entry(),
            padding: 0,
        });
        self.batcher
            .image(position, clip_shape.scissor, source, &mut self.resources.commands);
        self.cover(pixels);
    }

    /// The icon rasterized to the device pixels of `dest`, drawn texel for pixel like a glyph:
    /// tinted with `color` from the mask atlas, or in its own colors from the color atlas.
    fn icon(&mut self, icon: IconId, dest: &Rect, color: Color) {
        let scale = self.scale;
        let width = (dest.width * scale).round().max(1.0) as u32;
        let height = (dest.height * scale).round().max(1.0) as u32;
        let Some(placed) = self.glyph_atlases.icon(icon, width, height, self.media.icons) else {
            return;
        };
        let left = (dest.x * scale).round();
        let top = (dest.y * scale).round();
        let bounds = [left / scale, top / scale, width as f32 / scale, height as f32 / scale];
        let pixels = self.visible_pixels(bounds);
        if pixels.is_empty() {
            return;
        }
        let clip = self.active_clip();
        let position = self.resources.glyph_instances.len();
        self.resources.glyph_instances.push(GlyphInstance {
            bounds,
            color: premultiplied(color),
            atlas_origin: [placed.entry.x as f32, placed.entry.y as f32],
            clips_start: clip.rounded_start as u32,
            clips_count: clip.rounded_count as u32,
            is_color: u32::from(placed.atlas == AtlasKind::Color),
            transform: self.transform_entry(),
            padding: [0; 2],
        });
        self.batcher.glyph(
            position,
            clip.scissor,
            placed.atlas,
            placed.entry.page,
            &mut self.resources.commands,
        );
        self.batcher.glyphs_cover(pixels);
        self.cover(pixels);
    }

    /// Hands the content box to the application, clipped to it and to the clips in force.
    fn canvas(&mut self, canvas: CanvasId, dest: &Rect) {
        let bounds = [dest.x, dest.y, dest.width, dest.height];
        let pixels = self.visible_pixels(bounds);
        if pixels.is_empty() {
            return;
        }
        self.batcher.finish(&mut self.resources.commands);
        self.resources.commands.push(FrameCommand::Canvas {
            canvas,
            pixels,
            scissor: pixels,
            transform: self.transform_entry(),
        });
        self.cover(pixels);
    }

    /// The transform the items being encoded are placed by.
    fn transform(&self) -> Transform {
        self.open_groups
            .last()
            .map_or(Transform::IDENTITY, |group| group.transform)
    }

    fn transform_entry(&self) -> u32 {
        self.open_groups
            .last()
            .map_or(IDENTITY_TRANSFORM, |group| group.transform_entry)
    }

    /// Opens a group: its transform places everything inside, and a group with an effect
    /// draws everything inside into a picture of its own.
    fn push_group(&mut self, effects: &GroupEffects) {
        let transform = effects.transform;
        let transform_entry = if transform.is_identity() {
            IDENTITY_TRANSFORM
        } else {
            let position = self.resources.transforms.len() as u32;
            self.resources.transforms.push(TransformEntry::of(&transform));
            position
        };
        let pass = effects.needs_offscreen_pass().then(|| {
            self.batcher.finish(&mut self.resources.commands);
            let (filters_start, filters_count, blur) = self.filter_entries(effects.filter);
            let backdrop = (effects.backdrop_filter.start != effects.backdrop_filter.end).then(|| {
                let (filters_start, filters_count, blur) = self.filter_entries(effects.backdrop_filter);
                let reach = blur.map_or(0.0, |deviation| deviation * BLUR_REACH_PER_DEVIATION);
                let border_box = transform.bounds_of(&effects.frame.border_box);
                let pixels = PixelRect::covering(&border_box, self.scale)
                    .outset(reach.ceil() as usize)
                    .intersection(&self.target);
                BackdropPass {
                    pixels,
                    filters_start,
                    filters_count,
                    blur,
                    clip: ClipShape {
                        shape: RoundedRect::border_box(&effects.frame),
                        transform: transform_entry,
                        padding: [0; 3],
                    },
                }
            });
            let position = self.resources.groups.len();
            self.resources.groups.push(GroupPass {
                pixels: PixelRect::default(),
                opacity: effects.opacity,
                blend_mode: blend_mode_of(effects.mix_blend_mode),
                filters_start,
                filters_count,
                blur,
                backdrop,
            });
            self.resources.commands.push(FrameCommand::BeginGroup(position));
            position
        });
        self.open_groups.push(OpenGroup {
            transform,
            transform_entry,
            pass,
        });
    }

    /// Closes the innermost group. The picture of a group with an effect covers what was
    /// drawn into it and the reach of its blur; it is laid over the target the group began on.
    fn pop_group(&mut self) {
        let group = self.open_groups.pop().expect("a group is open when it is popped");
        let Some(pass) = group.pass else {
            return;
        };
        self.batcher.finish(&mut self.resources.commands);
        let group_pass = &mut self.resources.groups[pass];
        let reach = group_pass
            .blur
            .map_or(0.0, |deviation| deviation * BLUR_REACH_PER_DEVIATION);
        group_pass.pixels = group_pass
            .pixels
            .outset(reach.ceil() as usize)
            .intersection(&self.target);
        let pixels = group_pass.pixels;
        self.resources.commands.push(FrameCommand::EndGroup(pass));
        self.cover(pixels);
    }

    /// The color filters of a list, written to the filter buffer, and the deviation of its
    /// blur in device pixels; blurs in a row add up as Gaussians do.
    fn filter_entries(&mut self, filters: ItemRange) -> (u32, u32, Option<f32>) {
        let start = self.resources.filters.len() as u32;
        let mut variance = 0.0;
        for filter in self.display_list.filters(filters) {
            if let Filter::Blur(deviation) = filter {
                let deviation = deviation * self.scale;
                variance += deviation * deviation;
            } else {
                self.resources.filters.push(FilterEntry::of(*filter));
            }
        }
        let count = self.resources.filters.len() as u32 - start;
        (start, count, (variance > 0.0).then(|| variance.sqrt()))
    }

    /// Notes that the innermost group with a pass draws over `pixels`.
    fn cover(&mut self, pixels: PixelRect) {
        let Some(pass) = self.open_groups.iter().rev().find_map(|group| group.pass) else {
            return;
        };
        let group = &mut self.resources.groups[pass];
        group.pixels = group.pixels.union(&pixels);
    }

    /// The frame with every edge of its border box on the nearest edge between device pixels.
    /// Browsers paint boxes this way: a box laid out between pixels keeps sharp sides instead
    /// of blurring them over two pixels. A transformed box has no pixel grid to snap to.
    fn snapped(&self, frame: &BoxFrame) -> BoxFrame {
        if self.transform_entry() != IDENTITY_TRANSFORM {
            return *frame;
        }
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

    /// A rectangular clip on the pixel grid only narrows the scissor. A rounded or transformed
    /// one also joins the masks of the items inside it; the masks of an item are one run of the
    /// clip buffer, so the run of the enclosing clips is repeated in front of the new shape.
    fn push_clip(&mut self, frame: &BoxFrame) {
        let enclosing = self.active_clip();
        let shape = RoundedRect::padding_box(frame);
        let transform = self.transform_entry();
        let clip = if shape.is_rounded() || transform != IDENTITY_TRANSFORM {
            let rounded_start = self.resources.clips.len();
            for position in 0..enclosing.rounded_count {
                let enclosing_shape = self.resources.clips.get(enclosing.rounded_start + position);
                self.resources.clips.push(enclosing_shape);
            }
            self.resources.clips.push(ClipShape {
                shape,
                transform,
                padding: [0; 3],
            });
            let on_surface = self.transform().bounds_of(&shape.rect());
            ActiveClip {
                scissor: enclosing
                    .scissor
                    .intersection(&PixelRect::covering(&on_surface, self.scale)),
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
        !self.visible_pixels(bounds).is_empty()
    }

    /// The pixels of the surface the quad covers once transformed, as far as the clip in force
    /// lets them through.
    fn visible_pixels(&self, bounds: [f32; 4]) -> PixelRect {
        let [x, y, width, height] = bounds;
        let on_surface = self.transform().bounds_of(&Rect::new(x, y, width, height));
        PixelRect::covering(&on_surface, self.scale).intersection(&self.active_clip().scissor)
    }

    /// Appends the instance under the clip in force, unless the clip hides all of it.
    fn emit(&mut self, mut instance: RectInstance) {
        let pixels = self.visible_pixels(instance.bounds);
        if pixels.is_empty() {
            return;
        }
        let clip = self.active_clip();
        instance.clips_start = clip.rounded_start as u32;
        instance.clips_count = clip.rounded_count as u32;
        instance.transform = self.transform_entry();
        let position = self.resources.instances.len();
        self.resources.instances.push(instance);
        self.batcher
            .rect(position, pixels, clip.scissor, &mut self.resources.commands);
        self.cover(pixels);
    }

    /// Draws the glyphs of a run texel for pixel. The baseline lies on a row of device pixels;
    /// along it the pen keeps its place to a quarter of a pixel, and the glyph is taken from
    /// the atlas as rasterized for that quarter. Browsers place text this way.
    fn glyph_run(&mut self, origin: Point, font: FontId, glyphs: ItemRange, color: Color) {
        let clip = self.active_clip();
        let color = premultiplied(color);
        let scale = self.scale;
        let baseline = (origin.y * scale).round();
        let mut run_pixels = PixelRect::default();
        for glyph in self.display_list.glyphs(glyphs) {
            let (pen_pixel, subpixel_phase) = pen_position((origin.x + glyph.x) * scale);
            let placed = self.glyph_atlases.glyph(GlyphKey {
                font,
                glyph: glyph.glyph,
                scale: scale.to_bits(),
                subpixel_phase,
            });
            let Some(placed) = placed else {
                continue;
            };
            let extent = placed.entry.extent;
            let left = pen_pixel + extent.left as f32;
            let top = baseline + (glyph.y * scale).round() - extent.top as f32;
            let bounds = [
                left / scale,
                top / scale,
                extent.width as f32 / scale,
                extent.height as f32 / scale,
            ];
            let pixels = self.visible_pixels(bounds);
            if pixels.is_empty() {
                continue;
            }
            run_pixels = run_pixels.union(&pixels);
            let position = self.resources.glyph_instances.len();
            self.resources.glyph_instances.push(GlyphInstance {
                bounds,
                color,
                atlas_origin: [placed.entry.x as f32, placed.entry.y as f32],
                clips_start: clip.rounded_start as u32,
                clips_count: clip.rounded_count as u32,
                is_color: u32::from(placed.atlas == AtlasKind::Color),
                transform: self.transform_entry(),
                padding: [0; 2],
            });
            self.batcher.glyph(
                position,
                clip.scissor,
                placed.atlas,
                placed.entry.page,
                &mut self.resources.commands,
            );
        }
        self.batcher.glyphs_cover(run_pixels);
        self.cover(run_pixels);
    }

    /// The line of a text decoration lies on whole device pixels and is at least one of them
    /// thick: a hairline between pixels would turn gray.
    /// A rectangle of one color, its edges where the item puts them: the tree places the boxes
    /// of a selection and of a caret on device pixels itself.
    fn fill(&mut self, rect: &Rect, color: Color) {
        self.emit(RectInstance {
            bounds: self.bounds_of(rect),
            outer: RoundedRect {
                rect: [rect.x, rect.y, rect.width, rect.height],
                ..RoundedRect::zeroed()
            },
            colors: [premultiplied(color); 4],
            kind: RectKind::Fill as u32,
            ..RectInstance::zeroed()
        });
    }

    fn text_decoration(&mut self, rect: &Rect, color: Color) {
        let scale = self.scale;
        let snap = |edge: f32| (edge * scale).round() / scale;
        let left = snap(rect.x);
        let top = snap(rect.y);
        let thickness = (rect.height * scale).round().max(1.0) / scale;
        let line = Rect::new(left, top, snap(rect.right()) - left, thickness);
        self.emit(RectInstance {
            bounds: self.bounds_of(&line),
            outer: RoundedRect {
                rect: [line.x, line.y, line.width, line.height],
                ..RoundedRect::zeroed()
            },
            colors: [premultiplied(color); 4],
            kind: RectKind::Fill as u32,
            ..RectInstance::zeroed()
        });
    }
}

/// The number of a blend mode as the `BLEND_` constants of the shader give it.
fn blend_mode_of(mode: BlendMode) -> u32 {
    match mode {
        BlendMode::Normal => 0,
        BlendMode::Multiply => 1,
        BlendMode::Screen => 2,
        BlendMode::Overlay => 3,
        BlendMode::Darken => 4,
        BlendMode::Lighten => 5,
    }
}

/// The device pixel a pen at `pen_x` stands in, and which quarter of it.
fn pen_position(pen_x: f32) -> (f32, u8) {
    let phases = f32::from(SUBPIXEL_PHASES);
    let quarters = (pen_x * phases).round();
    let pixel = (quarters / phases).floor();
    (pixel, (quarters - pixel * phases) as u8)
}

#[cfg(test)]
mod tests {
    use super::pen_position;

    #[test]
    fn a_pen_keeps_its_place_to_a_quarter_of_a_pixel() {
        assert_eq!(pen_position(10.0), (10.0, 0));
        assert_eq!(pen_position(10.3), (10.0, 1));
        assert_eq!(pen_position(10.5), (10.0, 2));
        assert_eq!(pen_position(10.7), (10.0, 3));
    }

    #[test]
    fn a_pen_almost_on_the_next_pixel_stands_on_it() {
        assert_eq!(pen_position(10.9), (11.0, 0));
    }

    #[test]
    fn a_pen_left_of_the_surface_counts_its_quarters_rightwards_too() {
        assert_eq!(pen_position(-0.25), (-1.0, 3));
    }
}
