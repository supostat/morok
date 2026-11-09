mod support;

use morok_core::{DisplayItem, ElementId, LineBox, Point, Rect, Role, ShapedGlyph, Size};
use morok_css::SourceFile;
use morok_style::{Color, FontStyle, FontVariantNumeric, FontWeight};
use support::{ELLIPSIS, RequestedFont, Scene};

/// A font size of 20 pixels makes a character of the test font 12.5 pixels wide, with 18.75
/// pixels above the baseline and 6.25 below it: a content area of 25 pixels.
const TOKENS: &str = "--color-ink: #000000; --color-red: #FF0000; \
    --font-code: \"Plain Mono\", monospace; \
    --text-size: 20px; --text-small: 12px; --text-leading-flat: 1; --text-leading-loose: 2; \
    --text-weight-heavy: 800;";

const RED: Color = Color {
    red: 255,
    green: 0,
    blue: 0,
    alpha: 1.0,
};

const SIZED: &str = "font-size: var(--text-size);";
const LOOSE: &str = "font-size: var(--text-size); line-height: var(--text-leading-loose);";

fn scene(rules: &str) -> Scene {
    let tokens = SourceFile::new(
        "tokens.css",
        format!(":root {{ {TOKENS} }}\n@media (prefers-color-scheme: dark) {{ :root {{ {TOKENS} }} }}\n"),
    );
    Scene::with_tokens(&tokens, rules)
}

/// A scene with one block of the given classes under the root, and one text in the block.
fn text_in_block(rules: &str, block_classes: &[&str], text: &str) -> (Scene, ElementId, ElementId) {
    let mut scene = scene(rules);
    let block = scene.add(scene.root(), Role::Element, block_classes);
    let text = scene.add_text(block, text);
    scene.frame();
    (scene, block, text)
}

fn line(x: f32, y: f32, width: f32, height: f32, baseline: f32) -> LineBox {
    LineBox {
        rect: Rect::new(x, y, width, height),
        baseline,
    }
}

/// The glyph runs of the display list: where each starts, the characters its glyphs stand
/// for and their positions along the line.
fn glyph_runs(scene: &Scene) -> Vec<(Point, String, Vec<f32>)> {
    let display_list = scene.tree.display_list();
    display_list
        .items()
        .iter()
        .filter_map(|item| match item {
            DisplayItem::GlyphRun { origin, glyphs, .. } => {
                let glyphs: &[ShapedGlyph] = display_list.glyphs(*glyphs);
                Some((
                    *origin,
                    glyphs
                        .iter()
                        .map(|glyph| {
                            char::from_u32(u32::from(glyph.glyph)).expect("a glyph of the test font is a character")
                        })
                        .collect(),
                    glyphs.iter().map(|glyph| glyph.x).collect(),
                ))
            }
            _ => None,
        })
        .collect()
}

fn lines_of_text(scene: &Scene) -> Vec<String> {
    glyph_runs(scene)
        .into_iter()
        .map(|(_, characters, _)| characters)
        .collect()
}

/// What each text item of the display list is, in paint order.
fn text_item_kinds(scene: &Scene) -> Vec<&'static str> {
    scene
        .tree
        .display_list()
        .items()
        .iter()
        .filter_map(|item| match item {
            DisplayItem::GlyphRun { .. } => Some("glyphs"),
            DisplayItem::TextDecoration { .. } => Some("decoration"),
            _ => None,
        })
        .collect()
}

fn decorations(scene: &Scene) -> Vec<(Rect, Color)> {
    scene
        .tree
        .display_list()
        .items()
        .iter()
        .filter_map(|item| match item {
            DisplayItem::TextDecoration { rect, color } => Some((*rect, *color)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_line_box_is_as_high_as_the_line_height_with_the_content_area_in_its_middle() {
    let (scene, block, text) = text_in_block(&format!(".loose {{ {LOOSE} }}"), &["loose"], "word");

    assert_eq!(scene.tree.line_boxes(text), [line(0.0, 0.0, 50.0, 40.0, 26.25)]);
    assert_eq!(scene.border_box(text), Rect::new(0.0, 0.0, 800.0, 40.0));
    assert_eq!(
        scene.border_box(block).size(),
        Size {
            width: 800.0,
            height: 40.0
        }
    );
}

#[test]
fn a_line_height_below_the_content_area_pulls_the_baseline_up() {
    let (scene, _, text) = text_in_block(
        ".flat { font-size: var(--text-size); line-height: var(--text-leading-flat); }",
        &["flat"],
        "word",
    );

    assert_eq!(scene.tree.line_boxes(text), [line(0.0, 0.0, 50.0, 20.0, 16.25)]);
}

#[test]
fn a_normal_line_height_is_the_one_the_font_asks_for() {
    let (scene, _, text) = text_in_block(&format!(".sized {{ {SIZED} }}"), &["sized"], "word");

    assert_eq!(scene.tree.line_boxes(text), [line(0.0, 0.0, 50.0, 25.0, 18.75)]);
}

#[test]
fn lines_break_at_the_width_of_the_box_and_stack_at_the_line_height() {
    let (scene, block, text) = text_in_block(
        &format!(".narrow {{ width: 100px; {LOOSE} }}"),
        &["narrow"],
        "aaa bbb ccc ddd",
    );

    assert_eq!(
        scene.tree.line_boxes(text),
        [line(0.0, 0.0, 87.5, 40.0, 26.25), line(0.0, 40.0, 87.5, 40.0, 66.25)]
    );
    assert_eq!(lines_of_text(&scene), ["aaa bbb", "ccc ddd"]);
    assert_eq!(scene.border_box(text), Rect::new(0.0, 0.0, 100.0, 80.0));
    assert_eq!(
        scene.border_box(block).size(),
        Size {
            width: 100.0,
            height: 80.0
        }
    );
}

#[test]
fn a_text_without_characters_has_no_line_and_no_height() {
    let (scene, _, text) = text_in_block(&format!(".loose {{ {LOOSE} }}"), &["loose"], "");

    assert!(scene.tree.line_boxes(text).is_empty());
    assert_eq!(
        scene.border_box(text).size(),
        Size {
            width: 800.0,
            height: 0.0
        }
    );
    assert!(glyph_runs(&scene).is_empty());
}

#[test]
fn text_align_places_every_line_in_the_content_box() {
    let rules = format!(
        ".narrow {{ width: 100px; {SIZED} }}\n\
         .left {{ text-align: left; }} .start {{ text-align: start; }}\n\
         .right {{ text-align: right; }} .end {{ text-align: end; }}\n\
         .center {{ text-align: center; }}"
    );
    let offsets = [
        ("left", [0.0, 0.0]),
        ("start", [0.0, 0.0]),
        ("right", [12.5, 62.5]),
        ("end", [12.5, 62.5]),
        ("center", [6.25, 31.25]),
    ];

    for (alignment, expected) in offsets {
        let (scene, _, text) = text_in_block(&rules, &["narrow", alignment], "aaa bbb ccc");

        let line_starts: Vec<f32> = scene.tree.line_boxes(text).iter().map(|line| line.rect.x).collect();
        assert_eq!(line_starts, expected, "text-align: {alignment}");
    }
}

#[test]
fn a_line_wider_than_its_box_starts_at_the_edge_whatever_the_alignment() {
    let (scene, _, text) = text_in_block(
        &format!(".tight {{ width: 40px; text-align: center; white-space: nowrap; {SIZED} }}"),
        &["tight"],
        "aaaaaa",
    );

    assert_eq!(scene.tree.line_boxes(text), [line(0.0, 0.0, 75.0, 25.0, 18.75)]);
}

#[test]
fn letter_spacing_follows_every_character() {
    let (scene, _, text) = text_in_block(
        &format!(".spaced {{ letter-spacing: 2.5px; {SIZED} }}"),
        &["spaced"],
        "abcd",
    );

    assert_eq!(scene.tree.line_boxes(text), [line(0.0, 0.0, 60.0, 25.0, 18.75)]);
    assert_eq!(glyph_runs(&scene)[0].2, [0.0, 15.0, 30.0, 45.0]);
}

#[test]
fn collapsing_white_space_sets_words_one_space_apart() {
    let (scene, _, _) = text_in_block(&format!(".sized {{ {SIZED} }}"), &["sized"], "  one   two\n three ");

    assert_eq!(lines_of_text(&scene), ["one two three"]);
}

#[test]
fn nowrap_keeps_the_text_on_one_line_however_narrow_the_box() {
    let (scene, _, text) = text_in_block(
        &format!(".narrow {{ width: 50px; white-space: nowrap; {SIZED} }}"),
        &["narrow"],
        "one  two three",
    );

    assert_eq!(lines_of_text(&scene), ["one two three"]);
    assert_eq!(scene.tree.line_boxes(text), [line(0.0, 0.0, 162.5, 25.0, 18.75)]);
}

#[test]
fn pre_keeps_white_space_and_breaks_only_where_the_text_does() {
    let (scene, _, text) = text_in_block(
        &format!(".narrow {{ width: 30px; white-space: pre; {SIZED} }}"),
        &["narrow"],
        " a\n  bcd",
    );

    assert_eq!(lines_of_text(&scene), [" a", "  bcd"]);
    assert_eq!(
        scene.tree.line_boxes(text),
        [line(0.0, 0.0, 25.0, 25.0, 18.75), line(0.0, 25.0, 62.5, 25.0, 43.75)]
    );
}

#[test]
fn pre_wrap_keeps_white_space_and_wraps_lines_that_are_full() {
    let (scene, _, _) = text_in_block(
        &format!(".narrow {{ width: 50px; white-space: pre-wrap; {SIZED} }}"),
        &["narrow"],
        "aa  bb cc",
    );

    assert_eq!(
        scene.text_shaper.shaped_texts.last().map(String::as_str),
        Some("aa  bb cc")
    );
    assert_eq!(lines_of_text(&scene), ["aa", "bb", "cc"]);
}

const CELL: &str = "width: 100px; white-space: nowrap; text-overflow: ellipsis; font-size: var(--text-size);";

#[test]
fn a_line_that_overflows_a_clipping_box_ends_with_an_ellipsis() {
    let (scene, _, text) = text_in_block(
        &format!(".cell {{ overflow: hidden; {CELL} }}"),
        &["cell"],
        "abcdefghijkl",
    );

    assert_eq!(lines_of_text(&scene), [format!("abcdefg{ELLIPSIS}")]);
    assert_eq!(scene.tree.line_boxes(text), [line(0.0, 0.0, 100.0, 25.0, 18.75)]);
}

#[test]
fn a_line_that_fits_is_not_cut() {
    let (scene, _, _) = text_in_block(&format!(".cell {{ overflow: hidden; {CELL} }}"), &["cell"], "abcdefgh");

    assert_eq!(lines_of_text(&scene), ["abcdefgh"]);
}

#[test]
fn a_box_that_does_not_clip_cuts_nothing() {
    let (scene, _, _) = text_in_block(&format!(".cell {{ {CELL} }}"), &["cell"], "abcdefghijkl");

    assert_eq!(lines_of_text(&scene), ["abcdefghijkl"]);
}

#[test]
fn text_overflow_is_not_inherited() {
    let mut scene = scene(&format!(
        ".cell {{ overflow: hidden; {CELL} }}\n.inner {{ display: block; }}"
    ));
    let cell = scene.add(scene.root(), Role::Element, &["cell"]);
    let inner = scene.add(cell, Role::Element, &["inner"]);
    scene.add_text(inner, "abcdefghijkl");

    scene.frame();

    assert_eq!(lines_of_text(&scene), ["abcdefghijkl"]);
}

#[test]
fn a_box_that_starts_to_clip_cuts_the_text_it_holds() {
    let mut scene = scene(&format!(".cell {{ {CELL} }}\n.clipping {{ overflow: hidden; }}"));
    let cell = scene.add(scene.root(), Role::Element, &["cell"]);
    scene.add_text(cell, "abcdefghijkl");
    scene.frame();

    scene.add_classes(cell, &["clipping"]);
    scene.frame();

    assert_eq!(lines_of_text(&scene), [format!("abcdefg{ELLIPSIS}")]);
}

#[test]
fn an_underline_lies_under_the_glyphs_of_every_line() {
    let (scene, _, text) = text_in_block(
        &format!(".link {{ width: 100px; text-decoration: underline; color: var(--color-red); {SIZED} }}"),
        &["link"],
        "aaa bbb ccc",
    );

    assert_eq!(
        text_item_kinds(&scene),
        ["decoration", "glyphs", "decoration", "glyphs"]
    );
    let text_box = scene.border_box(text);
    assert_eq!(
        decorations(&scene),
        [
            (Rect::new(text_box.x, text_box.y + 18.75 + 2.5, 87.5, 1.25), RED),
            (Rect::new(text_box.x, text_box.y + 43.75 + 2.5, 37.5, 1.25), RED),
        ]
    );
}

#[test]
fn a_line_through_the_text_lies_over_its_glyphs() {
    let (scene, _, text) = text_in_block(
        &format!(".gone {{ text-decoration: line-through; color: var(--color-red); {SIZED} }}"),
        &["gone"],
        "abcd",
    );

    assert_eq!(text_item_kinds(&scene), ["glyphs", "decoration"]);
    let text_box = scene.border_box(text);
    assert_eq!(
        decorations(&scene),
        [(Rect::new(text_box.x, text_box.y + 18.75 - 7.5, 50.0, 1.25), RED)]
    );
}

#[test]
fn a_text_without_a_decoration_draws_its_glyphs_alone() {
    let (scene, _, _) = text_in_block(
        &format!(".plain {{ text-decoration: none; {SIZED} }}"),
        &["plain"],
        "abcd",
    );

    assert_eq!(text_item_kinds(&scene), ["glyphs"]);
}

#[test]
fn a_glyph_run_starts_on_the_baseline_of_its_line_in_the_color_of_the_text() {
    let (scene, _, text) = text_in_block(
        &format!(".note {{ padding: 10px 20px; color: var(--color-red); {LOOSE} }}"),
        &["note"],
        "ab",
    );

    assert_eq!(scene.border_box(text), Rect::new(20.0, 10.0, 760.0, 40.0));
    assert_eq!(
        glyph_runs(&scene),
        [(Point { x: 20.0, y: 36.25 }, String::from("ab"), vec![0.0, 12.5])]
    );
    let colors: Vec<Color> = scene
        .tree
        .display_list()
        .items()
        .iter()
        .filter_map(|item| match item {
            DisplayItem::GlyphRun { color, .. } => Some(*color),
            _ => None,
        })
        .collect();
    assert_eq!(colors, [RED]);
}

#[test]
fn the_padding_of_a_text_moves_its_lines_inside_its_box() {
    let mut scene = scene(&format!(".badge {{ padding: 5px 8px; {SIZED} }}"));
    let text = scene.add_text(scene.root(), "ab");
    scene.add_classes(text, &["badge"]);

    scene.frame();

    assert_eq!(scene.tree.line_boxes(text), [line(8.0, 5.0, 25.0, 25.0, 23.75)]);
    assert_eq!(
        scene.border_box(text).size(),
        Size {
            width: 800.0,
            height: 35.0
        }
    );
}

#[test]
fn the_shaper_is_asked_for_the_font_the_style_describes() {
    let (scene, _, _) = text_in_block(
        ".fancy { font-family: var(--font-code); font-size: var(--text-small); \
         font-weight: var(--text-weight-heavy); font-style: italic; font-variant-numeric: tabular-nums; }",
        &["fancy"],
        "12",
    );

    assert_eq!(
        scene.text_shaper.last_font,
        Some(RequestedFont {
            families: vec![String::from("Plain Mono"), String::from("monospace")],
            size: 12.0,
            weight: FontWeight(800),
            style: FontStyle::Italic,
            numeric: FontVariantNumeric::TabularNums,
        })
    );
}

#[test]
fn a_text_without_a_style_is_set_in_the_system_font_at_the_initial_size() {
    let (scene, _, _) = text_in_block(".plain { display: block; }", &["plain"], "ab");

    assert_eq!(
        scene.text_shaper.last_font,
        Some(RequestedFont {
            families: vec![String::from("system-ui")],
            size: 16.0,
            weight: FontWeight(400),
            style: FontStyle::Normal,
            numeric: FontVariantNumeric::Normal,
        })
    );
}

#[test]
fn text_properties_pass_from_an_element_to_the_text_below_it() {
    let mut scene = scene(&format!(
        ".page {{ text-align: right; letter-spacing: 2.5px; white-space: nowrap; {LOOSE} }}\n\
         .column {{ width: 100px; }}\n\
         .quiet {{ text-align: left; }}"
    ));
    scene.add_classes(scene.root(), &["page"]);
    let column = scene.add(scene.root(), Role::Element, &["column"]);
    let inherited = scene.add_text(column, "ab cd");
    let quiet = scene.add(scene.root(), Role::Element, &["column", "quiet"]);
    let overridden = scene.add_text(quiet, "ab cd");

    scene.frame();

    assert_eq!(scene.tree.line_boxes(inherited), [line(25.0, 0.0, 75.0, 40.0, 26.25)]);
    assert_eq!(scene.tree.line_boxes(overridden), [line(0.0, 0.0, 75.0, 40.0, 26.25)]);
}

#[test]
fn a_new_text_is_laid_out_and_drawn_in_place_of_the_old_one() {
    let (mut scene, _, text) = text_in_block(&format!(".narrow {{ width: 100px; {SIZED} }}"), &["narrow"], "aaa");

    scene.tree.set_text(text, "aaa bbb ccc");
    scene.frame();

    assert_eq!(lines_of_text(&scene), ["aaa bbb", "ccc"]);
    assert_eq!(scene.tree.line_boxes(text).len(), 2);
    assert!(!scene.tree.damage().is_empty());
}

#[test]
fn a_box_that_changes_its_width_breaks_the_lines_of_its_text_again() {
    let mut scene = scene(&format!(
        ".narrow {{ width: 100px; {SIZED} }}\n.wide {{ width: 300px; }}"
    ));
    let block = scene.add(scene.root(), Role::Element, &["narrow"]);
    scene.add_text(block, "aaa bbb ccc");
    scene.frame();
    assert_eq!(lines_of_text(&scene), ["aaa bbb", "ccc"]);

    scene.add_classes(block, &["wide"]);
    scene.frame();

    assert_eq!(lines_of_text(&scene), ["aaa bbb ccc"]);
}

#[test]
fn a_change_elsewhere_keeps_the_glyphs_without_shaping_them_again() {
    let mut scene = scene(&format!(
        ".note {{ {SIZED} }}\n.swatch {{ width: 10px; height: 10px; background-color: var(--color-ink); }}\n\
         .swatch:hover {{ background-color: var(--color-red); }}"
    ));
    let note = scene.add(scene.root(), Role::Element, &["note"]);
    scene.add_text(note, "kept text");
    let swatch = scene.add(scene.root(), Role::Element, &["swatch"]);
    scene.frame();
    let before = glyph_runs(&scene);
    let shaped_before = scene.text_shaper.shaped_texts.len();

    scene.tree.set_state(swatch, morok_style::States::HOVER, true);
    scene.frame();

    assert_eq!(glyph_runs(&scene), before);
    assert_eq!(scene.text_shaper.shaped_texts.len(), shaped_before);
}

#[test]
fn a_new_color_repaints_the_glyphs_without_shaping_them_again() {
    let mut scene = scene(&format!(".note {{ {SIZED} }}\n.alarm {{ color: var(--color-red); }}"));
    let note = scene.add(scene.root(), Role::Element, &["note"]);
    scene.add_text(note, "text");
    scene.frame();
    let shaped_before = scene.text_shaper.shaped_texts.len();

    scene.add_classes(note, &["alarm"]);
    scene.frame();

    let colors: Vec<Color> = scene
        .tree
        .display_list()
        .items()
        .iter()
        .filter_map(|item| match item {
            DisplayItem::GlyphRun { color, .. } => Some(*color),
            _ => None,
        })
        .collect();
    assert_eq!(colors, [RED]);
    assert_eq!(scene.text_shaper.shaped_texts.len(), shaped_before);
}
