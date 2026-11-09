use crate::paint::linear_gradient::LinearGradient;
use crate::paint::radial_gradient::RadialGradient;

#[derive(Clone, Debug, PartialEq)]
pub enum Gradient {
    Linear(LinearGradient),
    Radial(RadialGradient),
}
