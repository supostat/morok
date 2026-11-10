use crate::geometry::corners::Corners;
use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::geometry::size::Size;
use crate::paint::box_frame::BoxFrame;

/// A rectangle with rounded corners that a point is tested against.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HitShape {
    rect: Rect,
    /// The radii of each corner: across in the width, down in the height.
    radii: Corners<Size>,
}

impl HitShape {
    /// The border edge of a box: where the box itself can be hit.
    pub(crate) fn border_box(frame: &BoxFrame) -> Self {
        let circular = |radius: f32| Size {
            width: radius,
            height: radius,
        };
        Self {
            rect: frame.border_box,
            radii: Corners {
                top_left: circular(frame.radii.top_left),
                top_right: circular(frame.radii.top_right),
                bottom_right: circular(frame.radii.bottom_right),
                bottom_left: circular(frame.radii.bottom_left),
            },
        }
    }

    /// The padding edge of a box: where a box that clips its content lets the content be hit.
    pub(crate) fn padding_box(frame: &BoxFrame) -> Self {
        Self {
            rect: frame.padding_box(),
            radii: frame.padding_radii(),
        }
    }

    /// Whether the point lies in the shape. The left and top edges belong to it, the right and
    /// bottom ones to what lies beyond them, so two boxes side by side share no point.
    pub(crate) fn contains(&self, point: Point) -> bool {
        let rect = &self.rect;
        if point.x < rect.x || point.y < rect.y || point.x >= rect.right() || point.y >= rect.bottom() {
            return false;
        }
        let from_left = point.x - rect.x;
        let from_right = rect.right() - point.x;
        let from_top = point.y - rect.y;
        let from_bottom = rect.bottom() - point.y;
        is_inside_corner(self.radii.top_left, from_left, from_top)
            && is_inside_corner(self.radii.top_right, from_right, from_top)
            && is_inside_corner(self.radii.bottom_right, from_right, from_bottom)
            && is_inside_corner(self.radii.bottom_left, from_left, from_bottom)
    }
}

/// Whether a point at these distances from the two edges that meet in a corner lies inside
/// the ellipse that rounds the corner.
fn is_inside_corner(radii: Size, across: f32, down: f32) -> bool {
    if across >= radii.width || down >= radii.height {
        return true;
    }
    let across_from_center = (radii.width - across) / radii.width;
    let down_from_center = (radii.height - down) / radii.height;
    across_from_center * across_from_center + down_from_center * down_from_center <= 1.0
}

#[cfg(test)]
mod tests {
    use super::HitShape;
    use crate::geometry::corners::Corners;
    use crate::geometry::point::Point;
    use crate::geometry::rect::Rect;
    use crate::geometry::sides::Sides;
    use crate::paint::box_frame::BoxFrame;

    fn frame(radius: f32, border: f32) -> BoxFrame {
        BoxFrame {
            border_box: Rect::new(100.0, 100.0, 80.0, 40.0),
            border_widths: Sides::uniform(border),
            radii: Corners {
                top_left: radius,
                top_right: radius,
                bottom_right: radius,
                bottom_left: radius,
            },
        }
    }

    fn point(x: f32, y: f32) -> Point {
        Point { x, y }
    }

    #[test]
    fn the_left_and_top_edges_belong_to_a_shape_and_the_right_and_bottom_do_not() {
        let shape = HitShape::border_box(&frame(0.0, 0.0));

        assert!(shape.contains(point(100.0, 100.0)));
        assert!(shape.contains(point(179.5, 139.5)));
        assert!(!shape.contains(point(180.0, 120.0)));
        assert!(!shape.contains(point(120.0, 140.0)));
        assert!(!shape.contains(point(99.5, 120.0)));
    }

    #[test]
    fn a_point_in_the_cut_of_a_rounded_corner_is_outside() {
        let shape = HitShape::border_box(&frame(20.0, 0.0));

        assert!(!shape.contains(point(102.0, 102.0)));
        assert!(!shape.contains(point(178.0, 102.0)));
        assert!(!shape.contains(point(178.0, 138.0)));
        assert!(!shape.contains(point(102.0, 138.0)));
        assert!(shape.contains(point(107.0, 107.0)));
        assert!(shape.contains(point(140.0, 101.0)));
    }

    #[test]
    fn the_padding_edge_is_rounded_by_what_the_borders_leave_of_the_radius() {
        let rounded = HitShape::padding_box(&frame(20.0, 4.0));
        let square = HitShape::padding_box(&frame(4.0, 4.0));

        assert!(!rounded.contains(point(103.0, 120.0)));
        assert!(!rounded.contains(point(106.0, 106.0)));
        assert!(rounded.contains(point(110.0, 110.0)));
        assert!(square.contains(point(104.0, 104.0)));
    }
}
