use crate::geometry::corners::Corners;
use crate::geometry::rect::Rect;
use crate::geometry::sides::Sides;

/// The geometry of the box a display item belongs to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxFrame {
    /// The border box on the surface.
    pub border_box: Rect,
    /// The space each border takes; the padding box is the border box without it.
    pub border_widths: Sides<f32>,
    /// The radii of the corners of the border box, already scaled so that neighbours do not overlap.
    pub radii: Corners<f32>,
}

impl BoxFrame {
    pub fn padding_box(&self) -> Rect {
        self.border_box.expanded(Sides {
            top: -self.border_widths.top,
            right: -self.border_widths.right,
            bottom: -self.border_widths.bottom,
            left: -self.border_widths.left,
        })
    }
}

/// The used radii of a border box: when the radii of two neighbouring corners together exceed
/// the side they share, CSS scales every radius by the same factor until they fit.
pub(crate) fn fitted_radii(radii: Corners<f32>, border_box: &Rect) -> Corners<f32> {
    let side_factor = |side_length: f32, radii_sum: f32| {
        if radii_sum > side_length {
            side_length / radii_sum
        } else {
            1.0
        }
    };
    let factor = side_factor(border_box.width, radii.top_left + radii.top_right)
        .min(side_factor(border_box.width, radii.bottom_left + radii.bottom_right))
        .min(side_factor(border_box.height, radii.top_left + radii.bottom_left))
        .min(side_factor(border_box.height, radii.top_right + radii.bottom_right));
    Corners {
        top_left: radii.top_left * factor,
        top_right: radii.top_right * factor,
        bottom_right: radii.bottom_right * factor,
        bottom_left: radii.bottom_left * factor,
    }
}

#[cfg(test)]
mod tests {
    use super::{BoxFrame, fitted_radii};
    use crate::geometry::corners::Corners;
    use crate::geometry::rect::Rect;
    use crate::geometry::sides::Sides;

    fn uniform_radii(radius: f32) -> Corners<f32> {
        Corners {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    #[test]
    fn radii_that_fit_are_kept() {
        let border_box = Rect::new(0.0, 0.0, 100.0, 40.0);

        assert_eq!(fitted_radii(uniform_radii(8.0), &border_box), uniform_radii(8.0));
    }

    #[test]
    fn radii_that_overlap_are_scaled_by_the_tightest_side() {
        let border_box = Rect::new(0.0, 0.0, 100.0, 40.0);

        assert_eq!(fitted_radii(uniform_radii(40.0), &border_box), uniform_radii(20.0));
    }

    #[test]
    fn every_radius_is_scaled_by_the_same_factor() {
        let border_box = Rect::new(0.0, 0.0, 100.0, 100.0);
        let radii = Corners {
            top_left: 150.0,
            top_right: 50.0,
            bottom_right: 0.0,
            bottom_left: 0.0,
        };

        let fitted = fitted_radii(radii, &border_box);

        assert_eq!(
            fitted,
            Corners {
                top_left: 75.0,
                top_right: 25.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            }
        );
    }

    #[test]
    fn the_padding_box_is_the_border_box_without_the_borders() {
        let frame = BoxFrame {
            border_box: Rect::new(10.0, 20.0, 100.0, 50.0),
            border_widths: Sides {
                top: 1.0,
                right: 2.0,
                bottom: 3.0,
                left: 4.0,
            },
            radii: uniform_radii(0.0),
        };

        assert_eq!(frame.padding_box(), Rect::new(14.0, 21.0, 94.0, 46.0));
    }
}
