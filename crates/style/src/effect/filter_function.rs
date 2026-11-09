use crate::length::Length;

/// One function of a `filter` list, applied in the order written. An amount is a factor:
/// `1.0` leaves the picture as it is.
#[derive(Clone, Debug, PartialEq)]
pub enum FilterFunction {
    /// The standard deviation of the Gaussian.
    Blur(Length),
    Brightness(f32),
    Contrast(f32),
    /// From `0.0`, no change, to `1.0`, fully gray.
    Grayscale(f32),
    Saturate(f32),
    /// From `0.0`, fully transparent, to `1.0`, no change.
    Opacity(f32),
}
