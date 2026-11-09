use std::borrow::Cow;

use morok_core::{FontRequest, ShapedGlyph, ShapedText, TextExtent, TextRequest, TextShaper, WidthConstraint};
use morok_platform::CoreTextShaper;
use morok_style::{FontStyle, FontVariantNumeric, FontWeight};

static SYSTEM: [Cow<'static, str>; 1] = [Cow::Borrowed("system-ui")];
static SYSTEM_MONOSPACE: [Cow<'static, str>; 1] = [Cow::Borrowed("ui-monospace")];
static MISSING_THEN_MENLO: [Cow<'static, str>; 2] = [Cow::Borrowed("No Such Family"), Cow::Borrowed("Menlo")];
static MISSING: [Cow<'static, str>; 1] = [Cow::Borrowed("No Such Family")];
static GENERIC_MONOSPACE: [Cow<'static, str>; 1] = [Cow::Borrowed("monospace")];

const PANGRAM: &str = "The quick brown fox jumps over the lazy dog and keeps on running";

fn font(families: &'static [Cow<'static, str>], size: f32) -> FontRequest<'static> {
    FontRequest {
        families,
        size,
        weight: FontWeight(400),
        style: FontStyle::Normal,
        numeric: FontVariantNumeric::Normal,
    }
}

fn system_font() -> FontRequest<'static> {
    font(&SYSTEM, 16.0)
}

fn request<'a>(text: &'a str, font: FontRequest<'a>, width: WidthConstraint) -> TextRequest<'a> {
    TextRequest {
        text,
        font,
        letter_spacing: 0.0,
        width,
    }
}

fn width_of(shaper: &mut CoreTextShaper, text: &str, font: FontRequest<'_>) -> f32 {
    shaper.measure(&request(text, font, WidthConstraint::MaxContent)).width
}

fn shape(shaper: &mut CoreTextShaper, request: &TextRequest<'_>, ellipsis_width: Option<f32>) -> ShapedText {
    let mut typeset = ShapedText::default();
    shaper.shape(request, ellipsis_width, &mut typeset);
    typeset
}

/// The glyphs of a line, run after run.
fn line_glyphs(typeset: &ShapedText, line: usize) -> Vec<ShapedGlyph> {
    typeset
        .runs_of(&typeset.lines[line])
        .iter()
        .flat_map(|run| typeset.glyphs_of(run).iter().copied())
        .collect()
}

fn assert_close(actual: f32, expected: f32, tolerance: f32, what: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{what}: {actual} is not within {tolerance} of {expected}"
    );
}

#[test]
fn a_text_on_one_line_is_as_wide_as_its_glyphs_advance() {
    let mut shaper = CoreTextShaper::new();

    let extent = shaper.measure(&request("Hello", system_font(), WidthConstraint::MaxContent));
    let typeset = shape(
        &mut shaper,
        &request("Hello", system_font(), WidthConstraint::MaxContent),
        None,
    );

    assert_eq!(extent.line_count, 1);
    assert!(extent.width > 20.0 && extent.width < 60.0, "{extent:?}");
    assert_eq!(typeset.lines.len(), 1);
    assert_close(typeset.lines[0].width, extent.width, 0.01, "the typeset line");
    let glyphs = line_glyphs(&typeset, 0);
    assert_eq!(glyphs.len(), 5);
    assert!(glyphs.windows(2).all(|pair| pair[0].x < pair[1].x), "{glyphs:?}");
    assert!(glyphs.iter().all(|glyph| glyph.y.abs() < f32::EPSILON));
}

#[test]
fn a_text_without_characters_takes_no_line() {
    let mut shaper = CoreTextShaper::new();

    let extent = shaper.measure(&request("", system_font(), WidthConstraint::MaxContent));
    let typeset = shape(
        &mut shaper,
        &request("", system_font(), WidthConstraint::MaxContent),
        None,
    );

    assert_eq!(extent, TextExtent::default());
    assert!(typeset.lines.is_empty());
}

#[test]
fn lines_wrap_at_the_width_and_measuring_agrees_with_shaping() {
    let mut shaper = CoreTextShaper::new();
    let one_line = width_of(&mut shaper, PANGRAM, system_font());
    let width = one_line / 3.0;

    let extent = shaper.measure(&request(PANGRAM, system_font(), WidthConstraint::Definite(width)));
    let typeset = shape(
        &mut shaper,
        &request(PANGRAM, system_font(), WidthConstraint::Definite(width)),
        None,
    );

    assert!(typeset.lines.len() >= 3, "{} lines", typeset.lines.len());
    assert_eq!(extent.line_count, typeset.lines.len());
    let widest = typeset.lines.iter().map(|line| line.width).fold(0.0, f32::max);
    assert_close(extent.width, widest, 0.01, "the measured width");
    assert!(typeset.lines.iter().all(|line| line.width <= width + 0.01));
    let glyph_count: usize = (0..typeset.lines.len())
        .map(|line| line_glyphs(&typeset, line).len())
        .sum();
    assert!(glyph_count >= PANGRAM.split(' ').map(str::len).sum());
}

#[test]
fn a_line_exactly_as_wide_as_the_constraint_fits() {
    let mut shaper = CoreTextShaper::new();
    let one_line = width_of(&mut shaper, PANGRAM, system_font());

    let extent = shaper.measure(&request(PANGRAM, system_font(), WidthConstraint::Definite(one_line)));

    assert_eq!(extent.line_count, 1);
}

#[test]
fn a_word_wider_than_the_constraint_stays_whole() {
    let mut shaper = CoreTextShaper::new();
    let word = "Supercalifragilistic";
    let one_line = width_of(&mut shaper, word, system_font());

    let extent = shaper.measure(&request(word, system_font(), WidthConstraint::Definite(10.0)));

    assert_eq!(extent.line_count, 1);
    assert_close(extent.width, one_line, 0.01, "the overflowing word");
}

#[test]
fn the_min_content_width_is_the_widest_stretch_without_a_break() {
    let mut shaper = CoreTextShaper::new();
    let long_word = width_of(&mut shaper, "wider", system_font());

    let latin = shaper.measure(&request("a wider bit", system_font(), WidthConstraint::MinContent));
    let ideographs_on_a_line = width_of(&mut shaper, "中文字", system_font());
    let ideographs = shaper.measure(&request("中文字", system_font(), WidthConstraint::MinContent));

    assert_close(latin.width, long_word, 0.01, "the widest word");
    assert_eq!(latin.line_count, 3);
    assert_eq!(ideographs.line_count, 3, "a line may break between ideographs");
    assert_close(ideographs.width, ideographs_on_a_line / 3.0, 0.5, "one ideograph");
}

#[test]
fn a_line_break_in_the_text_always_ends_a_line() {
    let mut shaper = CoreTextShaper::new();

    let typeset = shape(
        &mut shaper,
        &request("one\ntwo\n\nfour", system_font(), WidthConstraint::MaxContent),
        None,
    );

    let glyphs_per_line: Vec<usize> = (0..typeset.lines.len())
        .map(|line| {
            line_glyphs(&typeset, line)
                .iter()
                .filter(|glyph| glyph.x >= 0.0)
                .count()
        })
        .collect();
    assert_eq!(typeset.lines.len(), 4, "{glyphs_per_line:?}");
    assert!(typeset.lines[2].width < 0.01, "an empty line has no width");
    assert!(typeset.lines[3].width > 0.0);
}

#[test]
fn the_width_of_a_line_leaves_out_the_white_space_it_ends_with() {
    let mut shaper = CoreTextShaper::new();

    let bare = width_of(&mut shaper, "word", system_font());
    let trailing = width_of(&mut shaper, "word   ", system_font());

    assert_close(trailing, bare, 0.01, "the line with trailing spaces");
}

#[test]
fn a_line_wider_than_the_ellipsis_width_is_cut_and_ends_with_an_ellipsis() {
    let mut shaper = CoreTextShaper::new();
    let whole = shape(
        &mut shaper,
        &request(PANGRAM, system_font(), WidthConstraint::MaxContent),
        None,
    );
    let ellipsis = shape(
        &mut shaper,
        &request("\u{2026}", system_font(), WidthConstraint::MaxContent),
        None,
    );
    let ellipsis_glyph = line_glyphs(&ellipsis, 0)[0].glyph;

    let cut = shape(
        &mut shaper,
        &request(PANGRAM, system_font(), WidthConstraint::MaxContent),
        Some(120.0),
    );

    assert_eq!(cut.lines.len(), 1);
    assert!(cut.lines[0].width <= 120.01, "{}", cut.lines[0].width);
    assert!(cut.lines[0].width > 90.0, "{}", cut.lines[0].width);
    let glyphs = line_glyphs(&cut, 0);
    assert!(glyphs.len() < line_glyphs(&whole, 0).len());
    assert_eq!(glyphs.last().map(|glyph| glyph.glyph), Some(ellipsis_glyph));
}

#[test]
fn a_line_that_fits_the_ellipsis_width_is_left_whole() {
    let mut shaper = CoreTextShaper::new();
    let whole = shape(
        &mut shaper,
        &request("Short", system_font(), WidthConstraint::MaxContent),
        None,
    );

    let kept = shape(
        &mut shaper,
        &request("Short", system_font(), WidthConstraint::MaxContent),
        Some(whole.lines[0].width),
    );

    assert_eq!(kept, whole);
}

#[test]
fn right_to_left_text_is_set_in_the_order_it_is_read() {
    let mut shaper = CoreTextShaper::new();
    let glyph_of = |shaper: &mut CoreTextShaper, letter: &str| {
        let alone = shape(
            shaper,
            &request(letter, system_font(), WidthConstraint::MaxContent),
            None,
        );
        line_glyphs(&alone, 0)[0].glyph
    };
    let alef = glyph_of(&mut shaper, "א");
    let gimel = glyph_of(&mut shaper, "ג");
    let latin_c = glyph_of(&mut shaper, "c");

    let mixed = shape(
        &mut shaper,
        &request("abc אבג", system_font(), WidthConstraint::MaxContent),
        None,
    );

    let glyphs = line_glyphs(&mixed, 0);
    let position = |glyph: u16| {
        glyphs
            .iter()
            .find(|candidate| candidate.glyph == glyph)
            .unwrap_or_else(|| panic!("the line holds the glyph {glyph}: {glyphs:?}"))
            .x
    };
    assert!(position(latin_c) < position(gimel), "the Latin word comes first");
    assert!(
        position(gimel) < position(alef),
        "the first Hebrew letter is the rightmost: {glyphs:?}"
    );
}

#[test]
fn characters_the_font_lacks_are_set_in_a_fallback_font() {
    let mut shaper = CoreTextShaper::new();

    let mixed = shape(
        &mut shaper,
        &request("A中", system_font(), WidthConstraint::MaxContent),
        None,
    );

    let runs = mixed.runs_of(&mixed.lines[0]);
    assert_eq!(runs.len(), 2, "{runs:?}");
    assert_ne!(runs[0].font, runs[1].font);
    assert!(runs.iter().all(|run| !mixed.glyphs_of(run).is_empty()));
}

#[test]
fn letter_spacing_follows_every_character() {
    let mut shaper = CoreTextShaper::new();
    let plain = width_of(&mut shaper, "abcd", system_font());

    let spaced = shaper
        .measure(&TextRequest {
            letter_spacing: 2.0,
            ..request("abcd", system_font(), WidthConstraint::MaxContent)
        })
        .width;

    assert_close(spaced, plain + 8.0, 0.5, "four characters spaced by two pixels");
}

#[test]
fn tabular_numbers_give_every_digit_one_advance() {
    let mut shaper = CoreTextShaper::new();
    let tabular = FontRequest {
        numeric: FontVariantNumeric::TabularNums,
        ..system_font()
    };

    let ones = width_of(&mut shaper, "1111", tabular);
    let zeros = width_of(&mut shaper, "0000", tabular);
    let proportional_ones = width_of(&mut shaper, "1111", system_font());

    assert_close(ones, zeros, 0.01, "four ones against four zeros");
    assert!(proportional_ones < ones, "{proportional_ones} against {ones}");
}

#[test]
fn a_heavier_weight_and_a_larger_size_set_wider_text() {
    let mut shaper = CoreTextShaper::new();
    let regular = width_of(&mut shaper, "Weight", system_font());

    let bold = width_of(
        &mut shaper,
        "Weight",
        FontRequest {
            weight: FontWeight(700),
            ..system_font()
        },
    );
    let doubled = width_of(&mut shaper, "Weight", font(&SYSTEM, 32.0));

    assert!(bold > regular, "{bold} against {regular}");
    assert_close(doubled, regular * 2.0, regular * 0.1, "the text at twice the size");
}

#[test]
fn an_italic_style_is_set_in_the_italic_face() {
    let mut shaper = CoreTextShaper::new();
    let italic = FontRequest {
        style: FontStyle::Italic,
        ..system_font()
    };

    let upright = shape(
        &mut shaper,
        &request("a", system_font(), WidthConstraint::MaxContent),
        None,
    );
    let slanted = shape(&mut shaper, &request("a", italic, WidthConstraint::MaxContent), None);

    assert_ne!(upright.runs[0].font, slanted.runs[0].font);
}

#[test]
fn the_first_family_the_system_has_is_the_font() {
    let mut shaper = CoreTextShaper::new();
    let narrow_and_wide = |shaper: &mut CoreTextShaper, families: &'static [Cow<'static, str>]| {
        (
            width_of(shaper, "iiii", font(families, 16.0)),
            width_of(shaper, "mmmm", font(families, 16.0)),
        )
    };

    let (menlo_narrow, menlo_wide) = narrow_and_wide(&mut shaper, &MISSING_THEN_MENLO);
    let (generic_narrow, generic_wide) = narrow_and_wide(&mut shaper, &GENERIC_MONOSPACE);
    let (system_monospace_narrow, system_monospace_wide) = narrow_and_wide(&mut shaper, &SYSTEM_MONOSPACE);
    let (system_narrow, system_wide) = narrow_and_wide(&mut shaper, &SYSTEM);
    let (missing_narrow, missing_wide) = narrow_and_wide(&mut shaper, &MISSING);

    assert_close(menlo_narrow, menlo_wide, 0.01, "Menlo after a family the system lacks");
    assert_close(generic_narrow, generic_wide, 0.01, "the generic monospace family");
    assert_close(
        system_monospace_narrow,
        system_monospace_wide,
        0.01,
        "the monospace system font",
    );
    assert!(system_narrow < system_wide);
    assert_close(
        missing_narrow,
        system_narrow,
        0.001,
        "the system font in place of a missing family",
    );
    assert_close(
        missing_wide,
        system_wide,
        0.001,
        "the system font in place of a missing family",
    );
}

/// The shaper reports what the font says; rounding to the pixels of a surface is the work of
/// line layout, which knows the scale of the surface.
#[test]
fn font_metrics_are_those_of_the_font_and_grow_with_the_size() {
    let mut shaper = CoreTextShaper::new();

    let metrics = shaper.font_metrics(&system_font());
    let doubled = shaper.font_metrics(&font(&SYSTEM, 32.0));

    assert!(metrics.ascent >= 12.0 && metrics.ascent <= 18.0, "{metrics:?}");
    assert!(metrics.descent >= 2.0 && metrics.descent <= 6.0, "{metrics:?}");
    assert!(
        metrics.underline_top > 0.0 && metrics.underline_top < metrics.descent + 2.0,
        "{metrics:?}"
    );
    assert!(
        metrics.line_through_top < 0.0 && -metrics.line_through_top < metrics.ascent,
        "{metrics:?}"
    );
    assert!(metrics.decoration_thickness > 0.0);
    assert_close(
        doubled.ascent,
        metrics.ascent * 2.0,
        0.01,
        "the ascent at twice the size",
    );
    assert_close(
        doubled.descent,
        metrics.descent * 2.0,
        0.01,
        "the descent at twice the size",
    );
}

#[test]
fn a_measurement_is_remembered_by_string_style_and_width() {
    let mut shaper = CoreTextShaper::new();
    let first = shaper.measure(&request(PANGRAM, system_font(), WidthConstraint::Definite(200.0)));
    assert_eq!(shaper.cached_measurements(), 1);

    let again = shaper.measure(&request(PANGRAM, system_font(), WidthConstraint::Definite(200.0)));
    assert_eq!(shaper.cached_measurements(), 1);
    assert_eq!(again, first);

    shaper.measure(&request(PANGRAM, system_font(), WidthConstraint::Definite(150.0)));
    shaper.measure(&request(PANGRAM, font(&SYSTEM, 20.0), WidthConstraint::Definite(200.0)));
    shaper.measure(&request(
        "Another text",
        system_font(),
        WidthConstraint::Definite(200.0),
    ));
    shaper.measure(&TextRequest {
        letter_spacing: 1.0,
        ..request(PANGRAM, system_font(), WidthConstraint::Definite(200.0))
    });
    assert_eq!(shaper.cached_measurements(), 5);
}
