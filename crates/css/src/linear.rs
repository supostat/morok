use std::borrow::Cow;

use morok_style::{Length, LengthPercentage, Pixels, Token, TokenTerm};

/// A length expression folded to `pixels + fraction × basis + Σ factor × token`.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Linear {
    pixels: f32,
    fraction: Option<f32>,
    tokens: Vec<TokenTerm>,
}

impl Linear {
    pub(crate) fn pixels(pixels: f32) -> Self {
        Self {
            pixels,
            ..Self::default()
        }
    }

    pub(crate) fn fraction(fraction: f32) -> Self {
        Self {
            fraction: Some(fraction),
            ..Self::default()
        }
    }

    pub(crate) fn token(token: Token<Pixels>) -> Self {
        Self {
            tokens: vec![TokenTerm { token, factor: 1.0 }],
            ..Self::default()
        }
    }

    pub(crate) fn plus(mut self, other: Self) -> Self {
        self.pixels += other.pixels;
        self.fraction = match (self.fraction, other.fraction) {
            (None, None) => None,
            (ours, theirs) => Some(ours.unwrap_or(0.0) + theirs.unwrap_or(0.0)),
        };
        for term in other.tokens {
            match self.tokens.iter_mut().find(|ours| ours.token == term.token) {
                Some(ours) => ours.factor += term.factor,
                None => self.tokens.push(term),
            }
        }
        self.tokens.retain(|term| term.factor != 0.0);
        self
    }

    pub(crate) fn scaled(mut self, factor: f32) -> Self {
        self.pixels *= factor;
        self.fraction = self.fraction.map(|fraction| fraction * factor);
        for term in &mut self.tokens {
            term.factor *= factor;
        }
        self.tokens.retain(|term| term.factor != 0.0);
        self
    }

    pub(crate) fn is_finite(&self) -> bool {
        self.pixels.is_finite()
            && self.fraction.is_none_or(f32::is_finite)
            && self.tokens.iter().all(|term| term.factor.is_finite())
    }

    pub(crate) fn has_percentage(&self) -> bool {
        self.fraction.is_some()
    }

    /// Whether the expression is one negative literal, such as `-4px` or `-10%`.
    pub(crate) fn is_negative_literal(&self) -> bool {
        self.tokens.is_empty() && (self.pixels < 0.0 || self.fraction.is_some_and(|fraction| fraction < 0.0))
    }

    pub(crate) fn into_length_percentage(self) -> LengthPercentage {
        LengthPercentage {
            pixels: Pixels(self.pixels),
            fraction: self.fraction,
            tokens: Cow::Owned(self.tokens),
        }
    }

    pub(crate) fn into_length(self) -> Length {
        debug_assert!(self.fraction.is_none(), "a length has no percentage part");
        Length {
            pixels: Pixels(self.pixels),
            tokens: Cow::Owned(self.tokens),
        }
    }
}
