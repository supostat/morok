use morok_style::Theme;

use crate::dirty::Dirty;
use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::paint::box_painter::BoxPainter;
use crate::paint::damage::Damage;
use crate::paint::display_item::DisplayItem;
use crate::paint::display_list::DisplayList;
use crate::paint::gradient_fill::GradientFill;
use crate::paint::item_range::ItemRange;
use crate::paint::paint_step::PaintStep;
use crate::paint::visual_bounds::visual_bounds;

/// The display list stage of a frame: writes the display list in paint order.
///
/// Only an element marked for painting rebuilds its items, and the area it drew before and the
/// area it draws now become damage. The items of every other element are taken over from the
/// previous list as they are.
#[derive(Debug)]
pub(crate) struct PaintPass<'a> {
    pub(crate) elements: &'a mut ElementArena,
    pub(crate) painter: BoxPainter<'a>,
    pub(crate) theme: &'a Theme,
    pub(crate) steps: &'a [PaintStep],
    pub(crate) previous: &'a DisplayList,
    pub(crate) display_list: &'a mut DisplayList,
    pub(crate) damage: &'a mut Damage,
}

impl PaintPass<'_> {
    /// Returns the number of elements whose items were rebuilt.
    pub(crate) fn run(mut self) -> usize {
        self.display_list.clear();
        let mut rebuilt_elements = 0;
        for &step in self.steps {
            match step {
                PaintStep::Box(id) if self.is_marked(id) => {
                    self.rebuild_box(id);
                    rebuilt_elements += 1;
                }
                PaintStep::Box(id) => {
                    self.elements[id].painted.box_items = self.keep(self.elements[id].painted.box_items);
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
        self.elements[id].dirty.contains(Dirty::PAINT)
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
        let bounds = visual_bounds(&element.style, &element.border_box, self.theme);
        self.damage.add(bounds);
        let box_items = self.items_since(start);
        let painted = &mut self.elements[id].painted;
        painted.is_painted = true;
        painted.visual_bounds = bounds;
        painted.box_items = box_items;
    }

    /// Takes items over from the previous list and returns where they are now. The stops of a
    /// gradient move with it.
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
                other => other,
            };
            self.display_list.items.push(item);
        }
        self.items_since(start)
    }
}
