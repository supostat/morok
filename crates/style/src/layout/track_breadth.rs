use crate::length_percentage::LengthPercentage;

/// One bound of a grid track size.
#[derive(Clone, Debug, PartialEq)]
pub enum TrackBreadth {
    LengthPercentage(LengthPercentage),
    /// A share of the free space, in `fr`.
    Flex(f32),
    Auto,
    MinContent,
    MaxContent,
}
