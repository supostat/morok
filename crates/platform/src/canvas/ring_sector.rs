use morok_core::Point;

/// The part of a ring between two angles: what a sunburst or a donut chart is drawn from.
/// Lengths are CSS pixels, the center is given from the top left corner of the canvas. An angle
/// is in radians and turns clockwise on the screen from the direction of the x axis, as the
/// angles of an SVG arc and of an HTML canvas do; the sector runs from `start_angle` to
/// `end_angle`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RingSector {
    pub center: Point,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub start_angle: f32,
    pub end_angle: f32,
}
