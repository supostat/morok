use morok_style::{BackdropFilterFunction, BlendMode, Declaration, FilterFunction, StyleSheet, Theme};

use crate::dirty::Dirty;
use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::geometry::rect::Rect;
use crate::geometry::sides::Sides;
use crate::geometry::transform::Transform;
use crate::paint::box_painter::BoxPainter;
use crate::paint::damage::Damage;
use crate::paint::display_item::DisplayItem;
use crate::paint::display_list::DisplayList;
use crate::paint::filter::Filter;
use crate::paint::gradient_fill::GradientFill;
use crate::paint::group_effects::GroupEffects;
use crate::paint::group_transform::{local_transform, scroll_transform};
use crate::paint::item_range::ItemRange;
use crate::paint::paint_step::PaintStep;
use crate::paint::visual_bounds::visual_bounds;
use crate::style::resolved_length::ResolvedLength;

/// The display list stage of a frame: writes the display list in paint order.
///
/// Only an element marked for painting rebuilds its items, and the area it drew before and the
/// area it draws now become damage. The items of every other element are taken over from the
/// previous list as they are. Inside a group whose element is marked, every element rebuilds:
/// the group's transform or effect may have changed, and its items are placed by the group.
#[derive(Debug)]
pub(crate) struct PaintPass<'a> {
    pub(crate) elements: &'a mut ElementArena,
    pub(crate) painter: BoxPainter<'a>,
    pub(crate) style_sheet: &'a StyleSheet,
    pub(crate) theme: &'a Theme,
    pub(crate) steps: &'a [PaintStep],
    pub(crate) previous: &'a DisplayList,
    pub(crate) display_list: &'a mut DisplayList,
    pub(crate) damage: &'a mut Damage,
    /// The groups enclosing the step being written, outermost first.
    pub(crate) open_groups: &'a mut Vec<OpenGroup>,
}

/// A group whose pop has not been written yet.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OpenGroup {
    element: ElementId,
    /// The transform of the group on the surface, enclosing groups included.
    transform: Transform,
    /// Whether every element inside rebuilds its items.
    repaints: bool,
    /// Whether an element inside rebuilt its items.
    has_rebuilt_content: bool,
    /// Everything drawn inside so far, on the surface.
    bounds: Rect,
    /// How far the filters of the group reach past its content.
    outset: f32,
    /// Whether the group moves the content of a `Scroll` element by its scroll, rather than
    /// placing and compositing an element with its content.
    is_scroll: bool,
}

impl PaintPass<'_> {
    /// Returns the number of elements whose items were rebuilt.
    pub(crate) fn run(mut self) -> usize {
        self.display_list.clear();
        self.open_groups.clear();
        let mut rebuilt_elements = 0;
        for &step in self.steps {
            match step {
                PaintStep::Box(id) if self.is_marked(id) => {
                    self.rebuild_box(id);
                    rebuilt_elements += 1;
                }
                PaintStep::Box(id) => {
                    self.elements[id].painted.box_items = self.keep(self.elements[id].painted.box_items);
                    self.cover(self.elements[id].painted.visual_bounds);
                }
                PaintStep::Outline(id) if self.is_marked(id) => {
                    let start = self.display_list.items.len();
                    BoxPainter::paint_outline(&self.elements[id], self.display_list);
                    self.elements[id].painted.outline_items = self.items_since(start);
                }
                PaintStep::Outline(id) => {
                    self.elements[id].painted.outline_items = self.keep(self.elements[id].painted.outline_items);
                }
                PaintStep::PushClip(id) => self.display_list.items.push(DisplayItem::PushClip {
                    frame: BoxPainter::frame(&self.elements[id]),
                }),
                PaintStep::PopClip => self.pop_clip(),
                PaintStep::PushGroup(id) => self.push_group(id),
                PaintStep::PopGroup => self.pop_group(),
                PaintStep::PushScroll(id) => self.push_scroll(id),
                PaintStep::PopScroll => self.pop_scroll(),
            }
        }
        for &step in self.steps {
            if let PaintStep::Box(id) = step {
                self.elements[id].dirty.remove(Dirty::PAINT);
            }
        }
        rebuilt_elements
    }

    fn is_marked(&self, id: ElementId) -> bool {
        self.elements[id].dirty.contains(Dirty::PAINT) || self.open_groups.last().is_some_and(|group| group.repaints)
    }

    /// The transform the items being written are placed by.
    fn transform(&self) -> Transform {
        self.open_groups
            .last()
            .map_or(Transform::IDENTITY, |group| group.transform)
    }

    /// Notes that the innermost open group drew inside `bounds`.
    fn cover(&mut self, bounds: Rect) {
        if let Some(group) = self.open_groups.last_mut() {
            group.bounds = if group.bounds.is_empty() {
                bounds
            } else {
                group.bounds.union(&bounds)
            };
        }
    }

    fn push_group(&mut self, id: ElementId) {
        let element = &self.elements[id];
        let transform = local_transform(element).then(&self.transform());
        let repaints = self.is_marked(id);
        if repaints && element.painted.is_painted {
            self.damage.add(element.painted.group_bounds);
        }
        let filter = self.resolve_filter(id);
        let backdrop_filter = self.resolve_backdrop_filter(id);
        let outset = self
            .display_list
            .filters(filter)
            .iter()
            .map(Filter::outset)
            .fold(0.0, f32::max);
        let element = &self.elements[id];
        self.display_list.items.push(DisplayItem::PushGroup {
            effects: GroupEffects {
                transform,
                opacity: element.style.effects.opacity,
                filter,
                backdrop_filter,
                mix_blend_mode: element.style.effects.mix_blend_mode,
                frame: BoxPainter::frame(element),
            },
        });
        self.open_groups.push(OpenGroup {
            element: id,
            transform,
            repaints,
            has_rebuilt_content: false,
            bounds: Rect::ZERO,
            outset,
            is_scroll: false,
        });
    }

    /// Opens the content of a `Scroll` element, moved by the offset its frame shows. A scroll
    /// that moved marks the element, so its content rebuilds at the place it shows now.
    fn push_scroll(&mut self, id: ElementId) {
        let element = &self.elements[id];
        let transform = scroll_transform(element).then(&self.transform());
        let repaints = self.is_marked(id);
        self.display_list.items.push(DisplayItem::PushGroup {
            effects: GroupEffects {
                transform,
                opacity: 1.0,
                filter: ItemRange::default(),
                backdrop_filter: ItemRange::default(),
                mix_blend_mode: BlendMode::Normal,
                frame: BoxPainter::frame(element),
            },
        });
        self.open_groups.push(OpenGroup {
            element: id,
            transform,
            repaints,
            has_rebuilt_content: false,
            bounds: Rect::ZERO,
            outset: 0.0,
            is_scroll: true,
        });
    }

    /// Closes the content of a `Scroll` element: what it drew counts as drawn by the group
    /// around it.
    fn pop_scroll(&mut self) {
        let group = self.open_groups.pop().expect("a scroll is open when it is popped");
        debug_assert!(
            group.is_scroll,
            "the innermost open group is a scroll when a scroll is popped"
        );
        self.display_list.items.push(DisplayItem::PopGroup);
        if let Some(parent) = self.open_groups.last_mut()
            && group.has_rebuilt_content
        {
            parent.has_rebuilt_content = true;
        }
        if !group.bounds.is_empty() {
            self.cover(group.bounds);
        }
    }

    /// Closes the open group. A group that composites its picture redraws it whole when
    /// anything inside changed, so the whole picture, as it was and as it is, is damage.
    fn pop_group(&mut self) {
        let group = self.open_groups.pop().expect("a group is open when it is popped");
        self.display_list.items.push(DisplayItem::PopGroup);
        let outset = Sides::uniform(group.outset);
        let bounds = group.transform.bounds_of(&group.bounds.expanded(outset));
        let painted = &mut self.elements[group.element].painted;
        let previous_bounds = painted.group_bounds;
        painted.group_bounds = bounds;
        let composites = self.elements[group.element].style.effects.needs_offscreen_pass();
        if composites && (group.repaints || group.has_rebuilt_content) {
            self.damage.add(previous_bounds);
            self.damage.add(bounds);
        }
        if group.has_rebuilt_content
            && let Some(parent) = self.open_groups.last_mut()
        {
            parent.has_rebuilt_content = true;
        }
        self.cover(bounds);
    }

    /// The `filter` list of the element, resolved into the filters of the frame.
    fn resolve_filter(&mut self, id: ElementId) -> ItemRange {
        let start = self.display_list.filters.len();
        let element = &self.elements[id];
        if let Some(source) = element.style.effects.filter {
            let Declaration::Filter(functions) = source.declaration(self.style_sheet, &element.inline_style) else {
                unreachable!("the filter of a style points at a filter declaration");
            };
            for function in functions.iter() {
                let filter = match function {
                    FilterFunction::Blur(radius) => Filter::Blur(ResolvedLength::pixels_of(radius, self.theme)),
                    FilterFunction::Brightness(amount) => Filter::Brightness(*amount),
                    FilterFunction::Contrast(amount) => Filter::Contrast(*amount),
                    FilterFunction::Grayscale(amount) => Filter::Grayscale(*amount),
                    FilterFunction::Saturate(amount) => Filter::Saturate(*amount),
                    FilterFunction::Opacity(amount) => Filter::Opacity(*amount),
                };
                self.display_list.filters.push(filter);
            }
        }
        self.filters_since(start)
    }

    /// The `backdrop-filter` list of the element, resolved into the filters of the frame.
    fn resolve_backdrop_filter(&mut self, id: ElementId) -> ItemRange {
        let start = self.display_list.filters.len();
        let element = &self.elements[id];
        if let Some(source) = element.style.effects.backdrop_filter {
            let Declaration::BackdropFilter(functions) = source.declaration(self.style_sheet, &element.inline_style)
            else {
                unreachable!("the backdrop filter of a style points at a backdrop filter declaration");
            };
            for function in functions.iter() {
                let filter = match function {
                    BackdropFilterFunction::Blur(radius) => Filter::Blur(ResolvedLength::pixels_of(radius, self.theme)),
                    BackdropFilterFunction::Brightness(amount) => Filter::Brightness(*amount),
                    BackdropFilterFunction::Saturate(amount) => Filter::Saturate(*amount),
                };
                self.display_list.filters.push(filter);
            }
        }
        self.filters_since(start)
    }

    fn filters_since(&self, start: usize) -> ItemRange {
        ItemRange {
            start: start as u32,
            end: self.display_list.filters.len() as u32,
        }
    }

    fn items_since(&self, start: usize) -> ItemRange {
        ItemRange {
            start: start as u32,
            end: self.display_list.items.len() as u32,
        }
    }

    /// Closes the open clip; a clip that enclosed no item is dropped instead.
    fn pop_clip(&mut self) {
        if matches!(self.display_list.items.last(), Some(DisplayItem::PushClip { .. })) {
            self.display_list.items.pop();
        } else {
            self.display_list.items.push(DisplayItem::PopClip);
        }
    }

    fn rebuild_box(&mut self, id: ElementId) {
        let element = &self.elements[id];
        if element.painted.is_painted {
            self.damage.add(element.painted.visual_bounds);
        }
        let start = self.display_list.items.len();
        self.painter.paint_box(element, self.display_list);
        let mut bounds = visual_bounds(element);
        if let Some(lines) = element.text_layout.bounds() {
            let lines_on_surface = Rect::new(
                element.border_box.x + lines.x,
                element.border_box.y + lines.y,
                lines.width,
                lines.height,
            );
            bounds = bounds.union(&lines_on_surface);
        }
        let bounds = self.transform().bounds_of(&bounds);
        self.damage.add(bounds);
        self.cover(bounds);
        if let Some(group) = self.open_groups.last_mut() {
            group.has_rebuilt_content = true;
        }
        let box_items = self.items_since(start);
        let painted = &mut self.elements[id].painted;
        painted.is_painted = true;
        painted.visual_bounds = bounds;
        painted.box_items = box_items;
    }

    /// Takes items over from the previous list and returns where they are now. The stops of a
    /// gradient and the glyphs of a run move with them.
    fn keep(&mut self, kept: ItemRange) -> ItemRange {
        let start = self.display_list.items.len();
        if kept.start == kept.end {
            return self.items_since(start);
        }
        for item in &self.previous.items[kept.as_range()] {
            let item = match *item {
                DisplayItem::Gradient { frame, gradient } => {
                    let stops_start = self.display_list.gradient_stops.len();
                    self.display_list
                        .gradient_stops
                        .extend_from_slice(self.previous.gradient_stops(gradient.stops));
                    DisplayItem::Gradient {
                        frame,
                        gradient: GradientFill {
                            shape: gradient.shape,
                            stops: ItemRange {
                                start: stops_start as u32,
                                end: self.display_list.gradient_stops.len() as u32,
                            },
                        },
                    }
                }
                DisplayItem::GlyphRun {
                    origin,
                    font,
                    glyphs,
                    color,
                } => {
                    let glyphs_start = self.display_list.glyphs.len();
                    self.display_list.glyphs.extend_from_slice(self.previous.glyphs(glyphs));
                    DisplayItem::GlyphRun {
                        origin,
                        font,
                        glyphs: ItemRange {
                            start: glyphs_start as u32,
                            end: self.display_list.glyphs.len() as u32,
                        },
                        color,
                    }
                }
                other => other,
            };
            self.display_list.items.push(item);
        }
        self.items_since(start)
    }
}
