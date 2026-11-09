#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TokenFamily {
    Color,
    Space,
    Radius,
    Font,
    Text,
    Shadow,
    Motion,
}

impl TokenFamily {
    pub(crate) const PREFIXES: &str = "color-, space-, radius-, font-, text-, shadow-, motion-";

    pub(crate) fn named(family: &str) -> Option<Self> {
        match family {
            "color" => Some(Self::Color),
            "space" => Some(Self::Space),
            "radius" => Some(Self::Radius),
            "font" => Some(Self::Font),
            "text" => Some(Self::Text),
            "shadow" => Some(Self::Shadow),
            "motion" => Some(Self::Motion),
            _ => None,
        }
    }
}
