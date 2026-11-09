use morok_style::ColorScheme;

use crate::rendering::Rendering;

/// The color schemes every case is rendered and compared under.
pub const COLOR_SCHEMES: [ColorScheme; 2] = [ColorScheme::Light, ColorScheme::Dark];

/// What one side drew of a case under both color schemes.
#[derive(Clone, Debug, PartialEq)]
pub struct CaseRendering {
    pub light: Rendering,
    pub dark: Rendering,
}

impl CaseRendering {
    pub fn under(&self, color_scheme: ColorScheme) -> &Rendering {
        match color_scheme {
            ColorScheme::Light => &self.light,
            ColorScheme::Dark => &self.dark,
        }
    }
}
