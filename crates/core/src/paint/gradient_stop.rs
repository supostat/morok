use morok_style::Color;

/// A color at a point of a gradient.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    /// The position as a fraction of the gradient line of a linear gradient, or of the horizontal
    /// radius of a radial one. Stops are ordered; a position may lie before 0 or after 1.
    pub offset: f32,
    pub color: Color,
}
