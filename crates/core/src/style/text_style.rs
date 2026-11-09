use std::borrow::Cow;

use morok_style::{
    FontFamilies, FontStyle, FontVariantNumeric, FontWeight, TextAlign, TextDecoration, TextOverflow, Theme, Token,
    WhiteSpace,
};

use crate::text::font_request::FontRequest;

/// The CSS initial `font-size`, `medium`.
const INITIAL_FONT_SIZE: f32 = 16.0;
const NORMAL_WEIGHT: FontWeight = FontWeight(400);
static SYSTEM_FONT_FAMILIES: [Cow<'static, str>; 1] = [Cow::Borrowed("system-ui")];

/// The part of a computed style that sets text. Every value but `text_overflow` passes from an
/// element to its children.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    /// None is the font of the system interface.
    pub font_family: Option<Token<FontFamilies>>,
    /// In CSS pixels.
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    /// The line height as a multiple of the font size; none is `normal`, the line height the
    /// font asks for.
    pub line_height: Option<f32>,
    pub letter_spacing: f32,
    pub text_align: TextAlign,
    pub white_space: WhiteSpace,
    pub font_variant_numeric: FontVariantNumeric,
    /// CSS draws the decoration of a box across the text of its descendants; here the value
    /// passes down to them like an inherited one.
    pub text_decoration: TextDecoration,
    /// Applies to the text children of the element; not inherited.
    pub text_overflow: TextOverflow,
}

impl TextStyle {
    pub(crate) const INITIAL: Self = Self {
        font_family: None,
        font_size: INITIAL_FONT_SIZE,
        font_weight: NORMAL_WEIGHT,
        font_style: FontStyle::Normal,
        line_height: None,
        letter_spacing: 0.0,
        text_align: TextAlign::Start,
        white_space: WhiteSpace::Normal,
        font_variant_numeric: FontVariantNumeric::Normal,
        text_decoration: TextDecoration::None,
        text_overflow: TextOverflow::Clip,
    };

    /// What a child starts from before its own declarations apply.
    #[must_use]
    pub(crate) fn inherited(&self) -> Self {
        Self {
            text_overflow: TextOverflow::Clip,
            ..*self
        }
    }

    /// The font the style asks a shaper for, its families read from the theme.
    pub(crate) fn font_request<'a>(&self, theme: &'a Theme) -> FontRequest<'a> {
        FontRequest {
            families: self
                .font_family
                .map_or(&SYSTEM_FONT_FAMILIES[..], |families| &theme.resolve(families).names),
            size: self.font_size,
            weight: self.font_weight,
            style: self.font_style,
            numeric: self.font_variant_numeric,
        }
    }
}
