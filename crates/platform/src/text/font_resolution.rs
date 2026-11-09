use std::ptr;

use morok_core::FontRequest;
use morok_style::{FontStyle, FontVariantNumeric, FontWeight};
use objc2_app_kit::NSFont;
use objc2_core_foundation::{CFDictionary, CFNumber, CFRetained, CFString, CFType, Type};
use objc2_core_text::{
    CTFont, CTFontDescriptor, CTFontSymbolicTraits, kCTFontFamilyNameAttribute, kCTFontTraitsAttribute,
    kCTFontWeightTrait,
};

/// The names CSS gives the fonts of the system interface.
const SYSTEM_INTERFACE_FAMILIES: [&str; 4] = ["system-ui", "-apple-system", "BlinkMacSystemFont", "ui-sans-serif"];
const SYSTEM_MONOSPACE_FAMILY: &str = "ui-monospace";
/// The fonts browsers on the Mac use for the generic families of CSS.
const GENERIC_FAMILIES: [(&str, &str); 3] = [("sans-serif", "Helvetica"), ("serif", "Times"), ("monospace", "Menlo")];

/// The weights of CSS on the scale Core Text measures weight by, from -1 to 1.
const WEIGHT_SCALE: [(u16, f64); 9] = [
    (100, -0.8),
    (200, -0.6),
    (300, -0.4),
    (400, 0.0),
    (500, 0.23),
    (600, 0.3),
    (700, 0.4),
    (800, 0.56),
    (900, 0.62),
];

/// The `number spacing` feature of a font and its `monospaced numbers` setting, by their
/// identifiers in the font feature registry.
const NUMBER_SPACING_FEATURE: i32 = 6;
const MONOSPACED_NUMBERS_SETTING: i32 = 0;

/// The font a request resolves to: the first family the system has, or the system interface
/// font when it has none of them.
pub(crate) fn resolve(request: &FontRequest<'_>) -> CFRetained<CTFont> {
    let size = f64::from(request.size);
    let weight = core_text_weight(request.weight);
    let mut font = request
        .families
        .iter()
        .find_map(|family| family_font(family, size, weight))
        .unwrap_or_else(|| system_font(&NSFont::systemFontOfSize_weight(size, weight)));
    if request.style == FontStyle::Italic {
        font = italic(&font).unwrap_or(font);
    }
    if request.numeric == FontVariantNumeric::TabularNums {
        font = with_tabular_numbers(&font);
    }
    font
}

fn family_font(family: &str, size: f64, weight: f64) -> Option<CFRetained<CTFont>> {
    if SYSTEM_INTERFACE_FAMILIES
        .iter()
        .any(|name| family.eq_ignore_ascii_case(name))
    {
        return Some(system_font(&NSFont::systemFontOfSize_weight(size, weight)));
    }
    if family.eq_ignore_ascii_case(SYSTEM_MONOSPACE_FAMILY) {
        return Some(system_font(&NSFont::monospacedSystemFontOfSize_weight(size, weight)));
    }
    let installed_name = GENERIC_FAMILIES
        .iter()
        .find(|(generic, _)| family.eq_ignore_ascii_case(generic))
        .map_or(family, |(_, installed)| installed);
    installed_font(installed_name, size, weight)
}

/// A font of `AppKit` is a font of Core Text: the two types are the same object.
fn system_font(font: &NSFont) -> CFRetained<CTFont> {
    // SAFETY: `NSFont` is toll-free bridged with `CTFont`.
    let font: &CTFont = unsafe { &*ptr::from_ref(font).cast::<CTFont>() };
    font.retain()
}

/// The font of an installed family, or none when the system does not have the family: asked
/// for a family it lacks, Core Text answers with a font of another family.
fn installed_font(family: &str, size: f64, weight: f64) -> Option<CFRetained<CTFont>> {
    let family_name = CFString::from_str(family);
    let weight = CFNumber::new_f64(weight);
    // SAFETY: the keys are constants of Core Text.
    let (family_key, traits_key, weight_key) =
        unsafe { (kCTFontFamilyNameAttribute, kCTFontTraitsAttribute, kCTFontWeightTrait) };
    let traits = CFDictionary::<CFString, CFNumber>::from_slices(&[weight_key], &[&*weight]);
    let keys: [&CFString; 2] = [family_key, traits_key];
    let values: [&CFType; 2] = [family_name.as_ref(), traits.as_ref()];
    let attributes = CFDictionary::<CFString, CFType>::from_slices(&keys, &values);
    // SAFETY: the attributes are a dictionary of font attributes; a null matrix is the identity.
    let (font, found_family) = unsafe {
        let descriptor = CTFontDescriptor::with_attributes(attributes.as_opaque());
        let font = CTFont::with_font_descriptor(&descriptor, size, ptr::null());
        let found_family = font.family_name();
        (font, found_family)
    };
    found_family.to_string().eq_ignore_ascii_case(family).then_some(font)
}

/// The italic face of the family of the font, when the family has one.
fn italic(font: &CTFont) -> Option<CFRetained<CTFont>> {
    // SAFETY: the font is valid; a null matrix keeps its own.
    unsafe {
        font.copy_with_symbolic_traits(
            0.0,
            ptr::null(),
            CTFontSymbolicTraits::ItalicTrait,
            CTFontSymbolicTraits::ItalicTrait,
        )
    }
}

fn with_tabular_numbers(font: &CTFont) -> CFRetained<CTFont> {
    let feature = CFNumber::new_i32(NUMBER_SPACING_FEATURE);
    let setting = CFNumber::new_i32(MONOSPACED_NUMBERS_SETTING);
    // SAFETY: the font is valid and the numbers identify a feature and one of its settings.
    unsafe {
        let descriptor = font.font_descriptor().copy_with_feature(&feature, &setting);
        CTFont::with_font_descriptor(&descriptor, font.size(), ptr::null())
    }
}

/// A weight between two steps of the scale lies as far between their values.
fn core_text_weight(weight: FontWeight) -> f64 {
    let css = weight
        .0
        .clamp(WEIGHT_SCALE[0].0, WEIGHT_SCALE[WEIGHT_SCALE.len() - 1].0);
    let heavier = WEIGHT_SCALE
        .iter()
        .position(|(step, _)| *step >= css)
        .expect("the weight is clamped to the scale");
    let (upper_css, upper) = WEIGHT_SCALE[heavier];
    if upper_css == css {
        return upper;
    }
    let (lower_css, lower) = WEIGHT_SCALE[heavier - 1];
    lower + (upper - lower) * f64::from(css - lower_css) / f64::from(upper_css - lower_css)
}

#[cfg(test)]
mod tests {
    use morok_style::FontWeight;

    use super::core_text_weight;

    fn thousandths(weight: u16) -> i64 {
        (core_text_weight(FontWeight(weight)) * 1000.0).round() as i64
    }

    #[test]
    fn the_steps_of_css_map_to_the_weights_of_the_system() {
        assert_eq!([thousandths(400), thousandths(600), thousandths(700)], [0, 300, 400]);
    }

    #[test]
    fn a_weight_between_two_steps_lies_between_their_values() {
        assert_eq!(thousandths(450), 115);
    }

    #[test]
    fn a_weight_off_the_scale_takes_its_nearest_end() {
        assert_eq!([thousandths(1), thousandths(1000)], [-800, 620]);
    }
}
