use bytemuck::{Pod, Zeroable};
use morok_core::{BoxFrame, Corners, Rect, Sides};

/// A rectangle whose corners are quarter ellipses, as the shader reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct RoundedRect {
    /// Origin and size.
    pub(crate) rect: [f32; 4],
    /// The radii of the corners along x, clockwise from the top left corner.
    pub(crate) horizontal_radii: [f32; 4],
    /// The radii of the same corners along y.
    pub(crate) vertical_radii: [f32; 4],
}

impl RoundedRect {
    pub(crate) fn border_box(frame: &BoxFrame) -> Self {
        let radii = corners_clockwise(frame.radii);
        Self {
            rect: origin_and_size(&frame.border_box),
            horizontal_radii: radii,
            vertical_radii: radii,
        }
    }

    /// The padding edge of a box: each corner loses the widths of the two borders that meet in
    /// it, and a corner whose radius does not reach past both of them is square.
    pub(crate) fn padding_box(frame: &BoxFrame) -> Self {
        let Sides {
            top,
            right,
            bottom,
            left,
        } = frame.border_widths;
        let corners = [
            inner_corner(frame.radii.top_left, left, top),
            inner_corner(frame.radii.top_right, right, top),
            inner_corner(frame.radii.bottom_right, right, bottom),
            inner_corner(frame.radii.bottom_left, left, bottom),
        ];
        Self {
            rect: origin_and_size(&frame.padding_box()),
            horizontal_radii: corners.map(|(horizontal, _)| horizontal),
            vertical_radii: corners.map(|(_, vertical)| vertical),
        }
    }

    pub(crate) fn rect(&self) -> Rect {
        let [x, y, width, height] = self.rect;
        Rect::new(x, y, width, height)
    }

    pub(crate) fn is_rounded(&self) -> bool {
        self.horizontal_radii.iter().any(|radius| *radius > 0.0)
    }

    /// The shape with every edge moved outwards by `distance`, or inwards by a negative one;
    /// the corners follow as CSS has those of a box shadow follow its spread.
    #[must_use]
    pub(crate) fn outset(&self, distance: f32) -> Self {
        let [x, y, width, height] = self.rect;
        let width = (width + 2.0 * distance).max(0.0);
        let height = (height + 2.0 * distance).max(0.0);
        let [center_x, center_y] = [x + self.rect[2] / 2.0, y + self.rect[3] / 2.0];
        let mut outset = Self {
            rect: [center_x - width / 2.0, center_y - height / 2.0, width, height],
            horizontal_radii: self.horizontal_radii.map(|radius| outset_radius(radius, distance)),
            vertical_radii: self.vertical_radii.map(|radius| outset_radius(radius, distance)),
        };
        for corner in 0..outset.horizontal_radii.len() {
            if outset.horizontal_radii[corner] <= 0.0 || outset.vertical_radii[corner] <= 0.0 {
                outset.horizontal_radii[corner] = 0.0;
                outset.vertical_radii[corner] = 0.0;
            }
        }
        outset
    }

    #[must_use]
    pub(crate) fn translated(&self, horizontal: f32, vertical: f32) -> Self {
        let [x, y, width, height] = self.rect;
        Self {
            rect: [x + horizontal, y + vertical, width, height],
            ..*self
        }
    }
}

fn origin_and_size(rect: &Rect) -> [f32; 4] {
    [rect.x, rect.y, rect.width, rect.height]
}

fn corners_clockwise(corners: Corners<f32>) -> [f32; 4] {
    [
        corners.top_left,
        corners.top_right,
        corners.bottom_right,
        corners.bottom_left,
    ]
}

fn inner_corner(radius: f32, horizontal_border: f32, vertical_border: f32) -> (f32, f32) {
    let horizontal = radius - horizontal_border;
    let vertical = radius - vertical_border;
    if horizontal > 0.0 && vertical > 0.0 {
        (horizontal, vertical)
    } else {
        (0.0, 0.0)
    }
}

/// A square corner stays square. A radius smaller than the distance grows by less than the
/// distance, on the cubic curve of CSS Backgrounds, so that a nearly square corner does not
/// turn round at once.
fn outset_radius(radius: f32, distance: f32) -> f32 {
    if distance <= 0.0 {
        return (radius + distance).max(0.0);
    }
    if radius >= distance {
        return radius + distance;
    }
    radius + distance * (1.0 + (radius / distance - 1.0).powi(3))
}

#[cfg(test)]
mod tests {
    use morok_core::{BoxFrame, Corners, Rect, Sides};

    use super::RoundedRect;

    fn frame(border_widths: Sides<f32>, radius: f32) -> BoxFrame {
        BoxFrame {
            border_box: Rect::new(10.0, 20.0, 100.0, 60.0),
            border_widths,
            radii: Corners {
                top_left: radius,
                top_right: radius,
                bottom_right: radius,
                bottom_left: radius,
            },
        }
    }

    fn border_box(radius: f32) -> RoundedRect {
        RoundedRect::border_box(&frame(Sides::uniform(0.0), radius))
    }

    fn circular(rect: [f32; 4], radius: f32) -> RoundedRect {
        RoundedRect {
            rect,
            horizontal_radii: [radius; 4],
            vertical_radii: [radius; 4],
        }
    }

    #[test]
    fn the_padding_edge_loses_the_borders_from_its_corners() {
        let widths = Sides {
            top: 2.0,
            right: 4.0,
            bottom: 6.0,
            left: 8.0,
        };

        let padding_box = RoundedRect::padding_box(&frame(widths, 10.0));

        assert_eq!(
            padding_box,
            RoundedRect {
                rect: [18.0, 22.0, 88.0, 52.0],
                horizontal_radii: [2.0, 6.0, 6.0, 2.0],
                vertical_radii: [8.0, 8.0, 4.0, 4.0],
            }
        );
    }

    #[test]
    fn a_corner_a_border_swallows_is_square() {
        let widths = Sides {
            top: 12.0,
            right: 1.0,
            bottom: 1.0,
            left: 1.0,
        };

        let padding_box = RoundedRect::padding_box(&frame(widths, 10.0));

        assert_eq!(
            padding_box,
            RoundedRect {
                rect: [11.0, 32.0, 98.0, 47.0],
                horizontal_radii: [0.0, 0.0, 9.0, 9.0],
                vertical_radii: [0.0, 0.0, 9.0, 9.0],
            }
        );
    }

    #[test]
    fn an_outset_moves_every_edge_and_grows_the_round_corners() {
        assert_eq!(border_box(8.0).outset(4.0), circular([6.0, 16.0, 108.0, 68.0], 12.0));
    }

    #[test]
    fn an_outset_keeps_square_corners_square() {
        assert!(!border_box(0.0).outset(4.0).is_rounded());
    }

    #[test]
    fn a_small_radius_grows_by_less_than_the_outset() {
        assert_eq!(border_box(2.0).outset(4.0), circular([6.0, 16.0, 108.0, 68.0], 5.5));
    }

    #[test]
    fn an_inset_shrinks_the_round_corners() {
        assert_eq!(border_box(8.0).outset(-3.0), circular([13.0, 23.0, 94.0, 54.0], 5.0));
    }

    #[test]
    fn an_inset_deeper_than_the_shape_leaves_no_area_and_no_corners() {
        assert_eq!(border_box(8.0).outset(-40.0), circular([50.0, 50.0, 20.0, 0.0], 0.0));
    }

    #[test]
    fn a_translation_moves_the_shape_and_keeps_its_corners() {
        assert_eq!(
            border_box(8.0).translated(5.0, -3.0),
            circular([15.0, 17.0, 100.0, 60.0], 8.0)
        );
    }
}
