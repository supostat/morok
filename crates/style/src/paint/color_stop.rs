use crate::length_percentage::LengthPercentage;
use crate::paint::color_value::ColorValue;

/// A stop of a gradient; a stop without a position is spread evenly between its neighbours, as in CSS.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorStop {
    pub color: ColorValue,
    pub position: Option<LengthPercentage>,
}
