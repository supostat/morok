/// The color scheme a theme is written for; matches the CSS `prefers-color-scheme` media feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorScheme {
    Light,
    Dark,
}

impl std::fmt::Display for ColorScheme {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Light => "light",
            Self::Dark => "dark",
        })
    }
}
