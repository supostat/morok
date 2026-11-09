use crate::token_family::TokenFamily;

/// The kind of value a token admits, fixed by its family and name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Color,
    Length,
    FontFamilies,
    Leading,
    FontWeight,
    Shadow,
    Motion,
}

impl TokenKind {
    pub(crate) fn of(family: TokenFamily, token_name: &str) -> Self {
        match family {
            TokenFamily::Color => Self::Color,
            TokenFamily::Font => Self::FontFamilies,
            TokenFamily::Text if token_name.starts_with("text-leading") => Self::Leading,
            TokenFamily::Text if token_name.starts_with("text-weight-") => Self::FontWeight,
            TokenFamily::Space | TokenFamily::Radius | TokenFamily::Text => Self::Length,
            TokenFamily::Shadow => Self::Shadow,
            TokenFamily::Motion => Self::Motion,
        }
    }

    pub(crate) fn expectation(self) -> &'static str {
        match self {
            Self::Color => "a color",
            Self::Length => "a length in px",
            Self::FontFamilies => "a list of font families",
            Self::Leading => "a unitless line height",
            Self::FontWeight => "an integer font weight from 1 to 1000",
            Self::Shadow => "a list of shadow layers",
            Self::Motion => "a duration or an easing",
        }
    }

    pub(crate) fn example(self) -> &'static str {
        match self {
            Self::Color => "#148F78 or rgba(17, 19, 24, 0.32)",
            Self::Length => "4px",
            Self::FontFamilies => "-apple-system, \"SF Pro Text\", sans-serif",
            Self::Leading => "1.45",
            Self::FontWeight => "500",
            Self::Shadow => "0 1px 2px rgba(17, 19, 24, 0.08)",
            Self::Motion => "120ms or cubic-bezier(0.2, 0, 0, 1)",
        }
    }
}
