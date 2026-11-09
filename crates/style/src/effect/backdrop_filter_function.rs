use crate::length::Length;

/// One function of a `backdrop-filter` list: what lies behind the element is filtered before
/// the element is drawn over it.
#[derive(Clone, Debug, PartialEq)]
pub enum BackdropFilterFunction {
    /// The standard deviation of the Gaussian.
    Blur(Length),
    Brightness(f32),
    Saturate(f32),
}
