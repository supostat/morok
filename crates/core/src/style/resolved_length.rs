use morok_style::{Length, LengthPercentage, Theme};

/// A `<length-percentage>` with its tokens resolved: `pixels + fraction × basis`.
///
/// A value written without a percentage has no fraction; it differs from a fraction of zero when
/// the basis is indefinite.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedLength {
    pub pixels: f32,
    pub fraction: Option<f32>,
}

impl ResolvedLength {
    pub const ZERO: Self = Self::pixels(0.0);

    pub const fn pixels(pixels: f32) -> Self {
        Self { pixels, fraction: None }
    }

    pub(crate) fn of(length: &LengthPercentage, theme: &Theme) -> Self {
        let token_pixels: f32 = length
            .tokens
            .iter()
            .map(|term| term.factor * theme.resolve(term.token).0)
            .sum();
        Self {
            pixels: length.pixels.0 + token_pixels,
            fraction: length.fraction,
        }
    }

    pub(crate) fn pixels_of(length: &Length, theme: &Theme) -> f32 {
        let token_pixels: f32 = length
            .tokens
            .iter()
            .map(|term| term.factor * theme.resolve(term.token).0)
            .sum();
        length.pixels.0 + token_pixels
    }

    /// The length against a definite basis.
    pub fn against(self, basis: f32) -> f32 {
        self.pixels + self.fraction.unwrap_or(0.0) * basis
    }
}
