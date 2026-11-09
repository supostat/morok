use morok_style::ColorScheme;
use objc2::rc::Retained;
use objc2_app_kit::{NSAppearance, NSAppearanceNameAqua, NSAppearanceNameDarkAqua};
use objc2_foundation::NSArray;

/// The appearance an application shows itself in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appearance {
    /// Whatever the user chose for the system; the application follows every change of it.
    System,
    Light,
    Dark,
}

impl Appearance {
    /// The appearance `AppKit` is told to use; none lets it follow the system.
    pub(crate) fn system_appearance(self) -> Option<Retained<NSAppearance>> {
        // SAFETY: the names are constants of `AppKit`.
        let name = match self {
            Self::System => return None,
            Self::Light => unsafe { NSAppearanceNameAqua },
            Self::Dark => unsafe { NSAppearanceNameDarkAqua },
        };
        Some(NSAppearance::appearanceNamed(name).expect("the two standard appearances exist"))
    }
}

/// The color scheme of an appearance of `AppKit`: its high contrast and vibrant variants count
/// as the light or the dark appearance they are built on.
pub(crate) fn color_scheme_of(appearance: &NSAppearance) -> ColorScheme {
    // SAFETY: the names are constants of `AppKit`.
    let (light, dark) = unsafe { (NSAppearanceNameAqua, NSAppearanceNameDarkAqua) };
    let best_match = appearance.bestMatchFromAppearancesWithNames(&NSArray::from_slice(&[light, dark]));
    if best_match.as_deref() == Some(dark) {
        ColorScheme::Dark
    } else {
        ColorScheme::Light
    }
}
