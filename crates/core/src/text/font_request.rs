use std::borrow::Cow;

use morok_style::{FontStyle, FontVariantNumeric, FontWeight};

/// The font a text is set in, as its style asks for it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontRequest<'a> {
    /// Family names in order of preference; generic names such as `system-ui` are among them.
    pub families: &'a [Cow<'static, str>],
    /// In CSS pixels.
    pub size: f32,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub numeric: FontVariantNumeric,
}
