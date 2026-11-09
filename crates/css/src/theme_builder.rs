use std::borrow::Cow;

use morok_style::{Color, Easing, FontFamilies, FontWeight, Leading, Milliseconds, Pixels, Shadow, Theme, Token};

use crate::token_id::TokenId;
use crate::token_value::TokenValue;

#[derive(Debug, Default)]
pub(crate) struct ThemeBuilder {
    colors: Vec<Color>,
    lengths: Vec<Pixels>,
    font_families: Vec<FontFamilies>,
    leadings: Vec<Leading>,
    font_weights: Vec<FontWeight>,
    shadows: Vec<Shadow>,
    durations: Vec<Milliseconds>,
    easings: Vec<Easing>,
}

impl ThemeBuilder {
    pub(crate) fn push(&mut self, value: TokenValue) -> TokenId {
        match value {
            TokenValue::Color(color) => TokenId::Color(appended(&mut self.colors, color)),
            TokenValue::Length(length) => TokenId::Length(appended(&mut self.lengths, length)),
            TokenValue::FontFamilies(families) => TokenId::FontFamilies(appended(&mut self.font_families, families)),
            TokenValue::Leading(leading) => TokenId::Leading(appended(&mut self.leadings, leading)),
            TokenValue::FontWeight(weight) => TokenId::FontWeight(appended(&mut self.font_weights, weight)),
            TokenValue::Shadow(shadow) => TokenId::Shadow(appended(&mut self.shadows, shadow)),
            TokenValue::Duration(duration) => TokenId::Duration(appended(&mut self.durations, duration)),
            TokenValue::Easing(easing) => TokenId::Easing(appended(&mut self.easings, easing)),
        }
    }

    pub(crate) fn build(self) -> Theme {
        Theme {
            colors: Cow::Owned(self.colors),
            lengths: Cow::Owned(self.lengths),
            font_families: Cow::Owned(self.font_families),
            leadings: Cow::Owned(self.leadings),
            font_weights: Cow::Owned(self.font_weights),
            shadows: Cow::Owned(self.shadows),
            durations: Cow::Owned(self.durations),
            easings: Cow::Owned(self.easings),
        }
    }
}

fn appended<V>(table: &mut Vec<V>, value: V) -> Token<V> {
    let index = u32::try_from(table.len()).expect("a theme table holds fewer than 2^32 tokens");
    table.push(value);
    Token::new(index)
}
