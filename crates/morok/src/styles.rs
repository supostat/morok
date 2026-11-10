use morok_components::styles::STYLE_SHEET;
use morok_components::tokens::{DARK, LIGHT};
use morok_style::{ColorScheme, StyleSheet, Theme};

/// The style tables an application is built with: its style sheet, in which the classes of
/// the kit have the IDs the components know them by, and the values of its tokens under each
/// color scheme.
#[derive(Clone, Debug, PartialEq)]
pub struct Styles {
    pub style_sheet: StyleSheet,
    pub light: Theme,
    pub dark: Theme,
}

impl Styles {
    /// The tables of the kit itself: the stylesheets of the components under the base theme.
    pub fn of_kit() -> Self {
        Self {
            style_sheet: STYLE_SHEET,
            light: LIGHT,
            dark: DARK,
        }
    }

    /// The values of the tokens under the color scheme.
    pub fn theme(&self, color_scheme: ColorScheme) -> &Theme {
        match color_scheme {
            ColorScheme::Light => &self.light,
            ColorScheme::Dark => &self.dark,
        }
    }
}
