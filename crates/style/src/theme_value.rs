use crate::color::Color;
use crate::easing::Easing;
use crate::font_families::FontFamilies;
use crate::font_weight::FontWeight;
use crate::leading::Leading;
use crate::milliseconds::Milliseconds;
use crate::pixels::Pixels;
use crate::shadow::Shadow;
use crate::theme::Theme;

/// A kind of value a design token can hold; names the theme table that stores it.
pub trait ThemeValue: Sized + 'static {
    fn table(theme: &Theme) -> &[Self];
}

impl ThemeValue for Color {
    fn table(theme: &Theme) -> &[Self] {
        &theme.colors
    }
}

impl ThemeValue for Pixels {
    fn table(theme: &Theme) -> &[Self] {
        &theme.lengths
    }
}

impl ThemeValue for FontFamilies {
    fn table(theme: &Theme) -> &[Self] {
        &theme.font_families
    }
}

impl ThemeValue for Leading {
    fn table(theme: &Theme) -> &[Self] {
        &theme.leadings
    }
}

impl ThemeValue for FontWeight {
    fn table(theme: &Theme) -> &[Self] {
        &theme.font_weights
    }
}

impl ThemeValue for Shadow {
    fn table(theme: &Theme) -> &[Self] {
        &theme.shadows
    }
}

impl ThemeValue for Milliseconds {
    fn table(theme: &Theme) -> &[Self] {
        &theme.durations
    }
}

impl ThemeValue for Easing {
    fn table(theme: &Theme) -> &[Self] {
        &theme.easings
    }
}
