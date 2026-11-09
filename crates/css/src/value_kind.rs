/// The kind of value a token holds once it is compiled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ValueKind {
    Color,
    Length,
    FontFamilies,
    Leading,
    FontWeight,
    Shadow,
    Duration,
    Easing,
}

impl ValueKind {
    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Color => "a color",
            Self::Length => "a length",
            Self::FontFamilies => "a list of font families",
            Self::Leading => "a line height",
            Self::FontWeight => "a font weight",
            Self::Shadow => "a shadow",
            Self::Duration => "a duration",
            Self::Easing => "an easing",
        }
    }
}
