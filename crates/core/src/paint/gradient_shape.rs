use crate::geometry::point::Point;

/// Where the colors of a gradient run, resolved against the padding box of its element.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientShape {
    /// The gradient line runs through the center of the padding box.
    Linear {
        /// Degrees clockwise from the upward direction: 0 points up and 90 to the right.
        angle: f32,
    },
    Radial {
        /// The center on the surface.
        center: Point,
        horizontal_radius: f32,
        vertical_radius: f32,
    },
}
