use std::borrow::Cow;

use morok_style::{LengthPercentage, LengthPercentageAuto, Pixels};

/// A length of `length` CSS pixels.
pub(crate) fn pixels(length: f32) -> LengthPercentageAuto {
    LengthPercentageAuto::LengthPercentage(LengthPercentage {
        pixels: Pixels(length),
        fraction: None,
        tokens: Cow::Borrowed(&[]),
    })
}
