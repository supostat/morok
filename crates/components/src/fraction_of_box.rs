use std::borrow::Cow;

use morok_style::{LengthPercentage, LengthPercentageAuto, Pixels};

/// A length that is `fraction` of the box it is measured against: what CSS writes as a
/// percentage.
pub(crate) fn fraction_of_box(fraction: f64) -> LengthPercentageAuto {
    #[expect(clippy::cast_possible_truncation, reason = "styles hold lengths in single precision")]
    let fraction = fraction as f32;
    LengthPercentageAuto::LengthPercentage(LengthPercentage {
        pixels: Pixels(0.0),
        fraction: Some(fraction),
        tokens: Cow::Borrowed(&[]),
    })
}
