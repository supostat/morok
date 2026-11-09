use crate::dirty::Dirty;
use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::role::Role;
use crate::style::positioning::Positioning;

/// Turns the boxes layout wrote, each relative to the element that positions it, into border
/// boxes on the surface, and marks for painting every element whose border box changed. A text
/// whose box changed its size has its lines laid out again: they are broken by its width.
#[derive(Debug)]
pub(crate) struct GeometryPass<'a> {
    pub(crate) elements: &'a mut ElementArena,
    pub(crate) repaints: bool,
}

impl GeometryPass<'_> {
    /// Returns whether an element must rebuild its display items.
    pub(crate) fn run(mut self, root: ElementId) -> bool {
        self.place(root, false);
        self.repaints
    }

    /// `positioner_moved` tells that an ancestor moved, so every box below it moved with it even
    /// though layout did not write it again.
    fn place(&mut self, id: ElementId, positioner_moved: bool) {
        let element = &self.elements[id];
        let written = element.dirty.intersects(Dirty::GEOMETRY.union(Dirty::GEOMETRY_BELOW));
        if !written && !positioner_moved {
            return;
        }
        let origin = self.positioner_origin(id);
        let layout = &element.layout;
        let border_box = Rect::new(
            origin.x + layout.location.x,
            origin.y + layout.location.y,
            layout.size.width,
            layout.size.height,
        );
        let previous = element.border_box;
        let moved = border_box.origin() != previous.origin();
        let element = &mut self.elements[id];
        element.dirty.remove(Dirty::GEOMETRY.union(Dirty::GEOMETRY_BELOW));
        if border_box != previous {
            element.border_box = border_box;
            element.dirty.insert(Dirty::PAINT);
            self.repaints = true;
        }
        if border_box.size() != previous.size() && element.role == Role::Text {
            self.elements.invalidate_text(id);
        }
        for position in 0..self.elements[id].children.len() {
            let child = self.elements[id].children[position];
            self.place(child, positioner_moved || moved);
        }
    }

    /// The corner the box of the element is relative to: that of its containing block when the
    /// element is out of flow, that of its parent otherwise. An out-of-flow element that layout
    /// skipped, hidden with its subtree, has no containing block and an empty box at its parent.
    fn positioner_origin(&self, id: ElementId) -> Point {
        let element = &self.elements[id];
        let is_out_of_flow = element.style.layout.positioning == Positioning::Absolute;
        let positioner = if is_out_of_flow {
            element.containing_block.or(element.parent)
        } else {
            element.parent
        };
        positioner.map_or(Point::default(), |positioner| {
            self.elements[positioner].border_box.origin()
        })
    }
}
