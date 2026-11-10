use crate::element::Element;
use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::geometry::point::Point;
use crate::input::hit_shape::HitShape;
use crate::paint::box_painter::BoxPainter;
use crate::paint::group_transform::{local_transform, scroll_transform};
use crate::paint::paint_step::PaintStep;

/// Finds the element a point of the surface shows.
///
/// The paint order is walked from the bottom up, as painting walks it: the last box that holds
/// the point is the one painted over the others there. A clip hides from the point what it
/// hides from the eye, and inside a group the point is taken back through the transform of the
/// group. A box is hit whatever it paints: a transparent box is there all the same.
#[derive(Debug)]
pub(crate) struct HitTest<'a> {
    pub(crate) elements: &'a ElementArena,
    /// The paint order of the last display list.
    pub(crate) steps: &'a [PaintStep],
    /// Scratch space for the point in the coordinates of every enclosing group.
    pub(crate) group_points: &'a mut Vec<Option<Point>>,
}

impl HitTest<'_> {
    pub(crate) fn element_at(self, point: Point) -> Option<ElementId> {
        self.group_points.clear();
        // None inside a group whose transform collapses the plane: nothing of it can be hit.
        let mut local_point = Some(point);
        let mut open_clips = 0_usize;
        // The outermost open clip the point lies outside of, as the number of clips open with it.
        let mut hiding_clip = None;
        let mut hit = None;
        for &step in self.steps {
            match step {
                PaintStep::Box(id) => {
                    if hiding_clip.is_none()
                        && let (Some(element), Some(local_point)) = (self.shown(id), local_point)
                        && HitShape::border_box(&BoxPainter::frame(element)).contains(local_point)
                    {
                        hit = Some(id);
                    }
                }
                PaintStep::Outline(_) => {}
                PaintStep::PushClip(id) => {
                    open_clips += 1;
                    if hiding_clip.is_none() && !self.shows_through_clip(id, local_point) {
                        hiding_clip = Some(open_clips);
                    }
                }
                PaintStep::PopClip => {
                    if hiding_clip == Some(open_clips) {
                        hiding_clip = None;
                    }
                    open_clips -= 1;
                }
                PaintStep::PushGroup(id) => {
                    self.group_points.push(local_point);
                    local_point = self.point_in_group(id, local_point);
                }
                PaintStep::PushScroll(id) => {
                    self.group_points.push(local_point);
                    local_point = self.point_in_scroll(id, local_point);
                }
                PaintStep::PopGroup | PaintStep::PopScroll => {
                    local_point = self
                        .group_points
                        .pop()
                        .expect("every group of the paint order is opened before it is closed");
                }
            }
        }
        hit
    }

    /// The element when it is still in the tree: the paint order is as old as the last display
    /// list, and an element may have been taken out since.
    fn shown(&self, id: ElementId) -> Option<&Element> {
        self.elements
            .contains(id)
            .then(|| &self.elements[id])
            .filter(|element| element.is_attached)
    }

    fn shows_through_clip(&self, id: ElementId, local_point: Option<Point>) -> bool {
        let (Some(element), Some(local_point)) = (self.shown(id), local_point) else {
            return false;
        };
        HitShape::padding_box(&BoxPainter::frame(element)).contains(local_point)
    }

    fn point_in_scroll(&self, id: ElementId, local_point: Option<Point>) -> Option<Point> {
        let into_content = scroll_transform(self.shown(id)?).inverse()?;
        Some(into_content.apply(local_point?))
    }

    fn point_in_group(&self, id: ElementId, local_point: Option<Point>) -> Option<Point> {
        let into_group = local_transform(self.shown(id)?).inverse()?;
        Some(into_group.apply(local_point?))
    }
}
