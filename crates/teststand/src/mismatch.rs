use std::fmt::{Display, Formatter, Result};

use crate::element_box::ElementBox;

/// One way a rendering differs from its etalon.
#[derive(Clone, Debug, PartialEq)]
pub enum Mismatch {
    /// The two trees hold different numbers of elements.
    ElementCount { etalon: usize, candidate: usize },
    /// An edge of the box lies further from the etalon than it may.
    ElementBox { etalon: ElementBox, candidate: ElementBox },
    /// The frames differ in size, in device pixels.
    ImageSize {
        etalon: (usize, usize),
        candidate: (usize, usize),
    },
    /// More pixels mismatch than may.
    MismatchingPixels { pixels: usize, area: usize },
    /// The mean color difference over the case is higher than it may be.
    MeanDifference { mean: f32 },
}

impl Display for Mismatch {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self {
            Self::ElementCount { etalon, candidate } => {
                write!(formatter, "the tree has {candidate} elements and the etalon {etalon}")
            }
            Self::ElementBox { etalon, candidate } => write!(
                formatter,
                "the box of `{}` is at {}, {} and {} by {}, the etalon has it at {}, {} and {} by {}",
                candidate.node,
                candidate.x,
                candidate.y,
                candidate.width,
                candidate.height,
                etalon.x,
                etalon.y,
                etalon.width,
                etalon.height
            ),
            Self::ImageSize { etalon, candidate } => write!(
                formatter,
                "the frame is {} by {} pixels and the etalon {} by {}",
                candidate.0, candidate.1, etalon.0, etalon.1
            ),
            Self::MismatchingPixels { pixels, area } => write!(
                formatter,
                "{pixels} of {area} pixels mismatch, {:.3} % of the case",
                *pixels as f32 / *area as f32 * 100.0
            ),
            Self::MeanDifference { mean } => {
                write!(formatter, "the mean color difference over the case is {mean:.3}")
            }
        }
    }
}
