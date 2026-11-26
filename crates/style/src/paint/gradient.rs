use crate::paint::linear_gradient::LinearGradient;
use crate::paint::radial_gradient::RadialGradient;

/// A gradient a background paints.
#[derive(Clone, Debug, PartialEq)]
pub enum Gradient {
    Linear(LinearGradient),
    Radial(RadialGradient),
}
