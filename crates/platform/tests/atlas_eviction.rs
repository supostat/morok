mod support;

use morok_core::Size;
use morok_platform::{AtlasUsage, FrameImage, Renderer, RendererOptions};
use support::Scene;

const SURFACE: Size = Size {
    width: 320.0,
    height: 80.0,
};

const RULES: &str = ".page { font-family: var(--font-ui); color: var(--color-ink); }\n\
    .small { font-size: var(--text-md); }\n\
    .large { font-size: var(--text-lg); }\n\
    .alarm { color: var(--color-red); }";

fn renderer(atlas_page_side: u32, atlas_page_budget: usize) -> Renderer {
    Renderer::with_options(&RendererOptions {
        atlas_page_side,
        atlas_page_budget,
    })
    .expect("this Mac runs Metal 4")
}

/// Draws one line of text under the root, set by the given classes.
fn draw(renderer: &Renderer, classes: &[&str], text: &str) -> FrameImage {
    let mut scene = Scene::new(RULES, SURFACE);
    scene.add_classes(scene.root(), &["page"]);
    let block = scene.add(scene.root(), classes);
    scene.add_text(block, text);
    scene.render(renderer, 1.0)
}

fn masks(renderer: &Renderer) -> AtlasUsage {
    renderer.atlas_statistics().mask
}

fn has_ink(image: &FrameImage) -> bool {
    image.pixels.chunks_exact(4).any(|pixel| pixel[3] > 0)
}

#[test]
fn glyphs_of_one_height_share_a_shelf_and_higher_ones_open_the_next() {
    let renderer = renderer(256, 4);

    let small = draw(&renderer, &["small"], "oooo");
    let after_small = masks(&renderer);
    draw(&renderer, &["large"], "OOOO");
    let after_large = masks(&renderer);

    assert!(has_ink(&small));
    assert_eq!((after_small.pages, after_small.shelves), (1, 1), "{after_small:?}");
    assert!(after_small.glyphs >= 1 && after_small.glyphs <= 4, "{after_small:?}");
    assert_eq!((after_large.pages, after_large.shelves), (1, 2), "{after_large:?}");
    assert_eq!(after_large.evicted_pages, 0);
}

#[test]
fn text_drawn_again_is_taken_from_the_atlas() {
    let renderer = renderer(256, 4);
    let first = draw(&renderer, &["small"], "Taken from the atlas");
    let rasterized = masks(&renderer).rasterized_glyphs;

    let second = draw(&renderer, &["small"], "Taken from the atlas");

    assert!(rasterized > 0);
    assert_eq!(masks(&renderer).rasterized_glyphs, rasterized);
    assert_eq!(second, first);
}

/// Every frame draws one glyph, so every frame uses one page: when a glyph finds no room, the
/// page the frame before drew from is the most recently used one, whatever the packing.
#[test]
fn at_its_budget_an_atlas_drops_its_least_recently_used_page_and_rasterizes_its_glyphs_again_on_demand() {
    let budget = 2;
    let renderer = renderer(64, budget);
    let letters: Vec<String> = ('a'..='z').chain('A'..='Z').map(String::from).collect();

    let mut kept_drawings: Vec<(&str, FrameImage)> = Vec::new();
    for letter in &letters {
        let drawing = draw(&renderer, &["large"], letter);
        if masks(&renderer).evicted_pages > 0 {
            break;
        }
        kept_drawings.push((letter, drawing));
    }
    let overflowed = masks(&renderer);
    assert_eq!(
        (overflowed.pages, overflowed.evicted_pages),
        (budget, 1),
        "the alphabet overflows the budget once: {overflowed:?}"
    );
    assert!(overflowed.glyphs <= kept_drawings.len(), "{overflowed:?}");

    let (drawn_before_the_overflow, _) = kept_drawings[kept_drawings.len() - 1];
    draw(&renderer, &["large"], drawn_before_the_overflow);
    assert_eq!(
        masks(&renderer).rasterized_glyphs,
        overflowed.rasterized_glyphs,
        "the page the frame before drew from is not the one dropped"
    );

    let came_back_unchanged = kept_drawings.iter().find_map(|(letter, first_drawing)| {
        let rasterized = masks(&renderer).rasterized_glyphs;
        let drawn_again = draw(&renderer, &["large"], letter);
        (masks(&renderer).rasterized_glyphs > rasterized).then(|| drawn_again == *first_drawing)
    });
    assert_eq!(
        came_back_unchanged,
        Some(true),
        "a glyph of the dropped page is rasterized again and looks as before"
    );
    assert_eq!(masks(&renderer).pages, budget);
}

#[test]
fn text_drawn_from_several_pages_looks_like_text_drawn_from_one() {
    let text = "Sphinx of black quartz";
    let roomy = renderer(1024, 4);
    let cramped = renderer(64, 64);

    let from_one_page = draw(&roomy, &["large"], text);
    let from_several_pages = draw(&cramped, &["large"], text);

    assert_eq!(masks(&roomy).pages, 1);
    assert!(masks(&cramped).pages > 1, "{:?}", masks(&cramped));
    assert!(has_ink(&from_one_page));
    assert_eq!(from_several_pages, from_one_page);
}

#[test]
fn a_glyph_is_tinted_with_the_color_of_its_text() {
    let renderer = renderer(256, 4);

    let image = draw(&renderer, &["large", "alarm"], "Red");

    let inked: Vec<&[u8]> = image.pixels.chunks_exact(4).filter(|pixel| pixel[3] > 0).collect();
    assert!(inked.len() > 100, "{} pixels", inked.len());
    assert!(
        inked
            .iter()
            .all(|pixel| pixel[0] == pixel[3] && pixel[1] == 0 && pixel[2] == 0)
    );
    assert!(inked.iter().any(|pixel| pixel[3] == 255), "the stems are fully covered");
}

#[test]
fn a_glyph_of_a_color_font_keeps_its_colors_in_the_color_atlas() {
    let renderer = renderer(256, 4);

    let image = draw(&renderer, &["large"], "\u{1F600}");

    let statistics = renderer.atlas_statistics();
    assert_eq!(
        (statistics.color.glyphs, statistics.mask.glyphs),
        (1, 0),
        "{statistics:?}"
    );
    assert!(
        image
            .pixels
            .chunks_exact(4)
            .any(|pixel| pixel[3] == 255 && pixel[0] > pixel[2].saturating_add(50)),
        "the face of the emoji is yellow"
    );
}

#[test]
fn text_lies_on_its_baseline_inside_its_line_box() {
    let renderer = renderer(256, 4);
    let mut scene = Scene::new(RULES, SURFACE);
    scene.add_classes(scene.root(), &["page"]);
    let block = scene.add(scene.root(), &["large"]);
    let text = scene.add_text(block, "Hxk");

    let image = scene.render(&renderer, 1.0);

    let line = scene.tree.line_boxes(text)[0];
    let inked_rows: Vec<usize> = (0..image.height)
        .filter(|row| (0..image.width).any(|column| image.pixel(column, *row)[3] > 0))
        .collect();
    let (first_row, last_row) = (inked_rows[0] as f32, inked_rows[inked_rows.len() - 1] as f32);
    assert!(
        first_row >= line.rect.y,
        "the ink starts at row {first_row}, the line at {}",
        line.rect.y
    );
    assert!(
        (last_row + 1.0 - line.baseline).abs() <= 1.0,
        "letters without descenders end on the baseline {}: the last inked row is {last_row}",
        line.baseline
    );
    let inked_columns: Vec<usize> = (0..image.width)
        .filter(|column| (0..image.height).any(|row| image.pixel(*column, row)[3] > 0))
        .collect();
    let ink_width = (inked_columns[inked_columns.len() - 1] - inked_columns[0]) as f32;
    assert!(
        (ink_width - line.rect.width).abs() <= 6.0,
        "the ink is {ink_width} wide, the line {}",
        line.rect.width
    );
}

#[test]
fn a_glyph_stands_upright_and_reads_left_to_right() {
    let renderer = renderer(256, 4);

    let image = draw(&renderer, &["large"], "L");

    let ink_in_row = |row: usize| {
        (0..image.width)
            .filter(|column| image.pixel(*column, row)[3] > 127)
            .count()
    };
    let ink_in_column = |column: usize| {
        (0..image.height)
            .filter(|row| image.pixel(column, *row)[3] > 127)
            .count()
    };
    let inked_rows: Vec<usize> = (0..image.height).filter(|row| ink_in_row(*row) > 0).collect();
    let inked_columns: Vec<usize> = (0..image.width).filter(|column| ink_in_column(*column) > 0).collect();
    let (top, bottom) = (inked_rows[0], inked_rows[inked_rows.len() - 1]);
    let (left, right) = (inked_columns[0], inked_columns[inked_columns.len() - 1]);
    assert!(
        ink_in_row(bottom) > ink_in_row(top) * 2,
        "the foot of an L is its widest row: {} pixels against {} at the top",
        ink_in_row(bottom),
        ink_in_row(top)
    );
    assert!(
        ink_in_column(left) > ink_in_column(right) * 2,
        "the stem of an L is its left edge: {} pixels against {} at the right",
        ink_in_column(left),
        ink_in_column(right)
    );
}
