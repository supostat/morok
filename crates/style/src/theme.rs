use std::borrow::Cow;

use crate::color::Color;
use crate::easing::Easing;
use crate::font_families::FontFamilies;
use crate::font_weight::FontWeight;
use crate::leading::Leading;
use crate::milliseconds::Milliseconds;
use crate::pixels::Pixels;
use crate::shadow::Shadow;
use crate::theme_value::ThemeValue;
use crate::token::Token;

/// The values of every design token under one color scheme, one table per kind of value.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub colors: Cow<'static, [Color]>,
    pub lengths: Cow<'static, [Pixels]>,
    pub font_families: Cow<'static, [FontFamilies]>,
    pub leadings: Cow<'static, [Leading]>,
    pub font_weights: Cow<'static, [FontWeight]>,
    pub shadows: Cow<'static, [Shadow]>,
    pub durations: Cow<'static, [Milliseconds]>,
    pub easings: Cow<'static, [Easing]>,
}

impl Theme {
    pub fn resolve<V: ThemeValue>(&self, token: Token<V>) -> &V {
        &V::table(self)[token.index()]
    }
}
