use morok_style::ColorScheme;

/// What the user asked of the system that changes how a surface shows its content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceSettings {
    pub color_scheme: ColorScheme,
    /// Whether "Reduce motion" is on in the accessibility settings of the system.
    pub reduces_motion: bool,
}
