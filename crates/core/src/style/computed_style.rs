use crate::style::effect_style::EffectStyle;
use crate::style::layout_style::LayoutStyle;
use crate::style::motion_style::MotionStyle;
use crate::style::paint_style::{INITIAL_COLOR, PaintStyle};
use crate::style::text_style::TextStyle;

/// The style of one element after the cascade, inheritance and the active theme: every token is
/// resolved to its value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputedStyle {
    pub layout: LayoutStyle,
    pub paint: PaintStyle,
    pub text: TextStyle,
    pub effects: EffectStyle,
    pub(crate) motion: MotionStyle,
}

impl ComputedStyle {
    pub(crate) const INITIAL: Self = Self {
        layout: LayoutStyle::INITIAL,
        paint: PaintStyle::initial(INITIAL_COLOR),
        text: TextStyle::INITIAL,
        effects: EffectStyle::INITIAL,
        motion: MotionStyle::INITIAL,
    };
}
