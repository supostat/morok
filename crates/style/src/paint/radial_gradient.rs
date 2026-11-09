use std::borrow::Cow;

use crate::length_percentage::LengthPercentage;
use crate::paint::color_stop::ColorStop;
use crate::paint::radial_extent::RadialExtent;
use crate::paint::radial_shape::RadialShape;

/// The center is measured from the top left corner of the box.
#[derive(Clone, Debug, PartialEq)]
pub struct RadialGradient {
    pub shape: RadialShape,
    pub extent: RadialExtent,
    pub center_x: LengthPercentage,
    pub center_y: LengthPercentage,
    pub stops: Cow<'static, [ColorStop]>,
}
