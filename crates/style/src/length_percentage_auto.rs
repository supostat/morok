use crate::length_percentage::LengthPercentage;

/// A size, margin or inset. For `max-width` and `max-height` the automatic value is the CSS `none`.
#[derive(Clone, Debug, PartialEq)]
pub enum LengthPercentageAuto {
    Auto,
    LengthPercentage(LengthPercentage),
}
