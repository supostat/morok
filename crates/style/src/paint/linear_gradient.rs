use std::borrow::Cow;

use crate::paint::color_stop::ColorStop;
use crate::paint::gradient_direction::GradientDirection;

#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    pub direction: GradientDirection,
    pub stops: Cow<'static, [ColorStop]>,
}
