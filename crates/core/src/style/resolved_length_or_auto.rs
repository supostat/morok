use morok_style::{LengthPercentageAuto, Theme};

use crate::style::resolved_length::ResolvedLength;

/// A size, margin or inset with its tokens resolved. For `max-width` and `max-height` the automatic
/// value is the CSS `none`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResolvedLengthOrAuto {
    Auto,
    Length(ResolvedLength),
}

impl ResolvedLengthOrAuto {
    pub const ZERO: Self = Self::Length(ResolvedLength::ZERO);

    pub(crate) fn of(length: &LengthPercentageAuto, theme: &Theme) -> Self {
        match length {
            LengthPercentageAuto::Auto => Self::Auto,
            LengthPercentageAuto::LengthPercentage(length) => Self::Length(ResolvedLength::of(length, theme)),
        }
    }
}
