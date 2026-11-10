use morok_core::{ElementId, Size};
use morok_css::SourceFile;
use morok_style::States;

use crate::case::Case;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;

pub(crate) const NAME: &str = "primitives";

const STYLESHEET_PATH: &str = "crates/teststand/suites/primitives.css";
const STYLESHEET: &str = include_str!("../../suites/primitives.css");

const PANGRAM: &str = "The quick brown fox jumps over the lazy dog";
const PARAGRAPH: &str = "Morok draws every glyph itself, so each line has to break where a browser breaks it.";

/// The primitives every component is made of: boxes, flex and grid layouts, and text.
pub(crate) fn suite() -> Suite {
    Suite {
        name: NAME,
        stylesheets: vec![SourceFile::new(STYLESHEET_PATH, STYLESHEET)],
        cases: vec![
            case("boxes", 392.0, 224.0, boxes),
            case("shadows", 496.0, 176.0, shadows),
            case("gradients", 328.0, 192.0, gradients),
            case("overlap", 320.0, 220.0, overlap),
            case("flex", 360.0, 332.0, flex),
            case("grid", 360.0, 228.0, grid),
            case("text-sizes", 420.0, 264.0, text_sizes),
            case("text-weights", 380.0, 160.0, text_weights),
            case("text-leading", 264.0, 316.0, text_leading),
            case("text-letter-spacing", 420.0, 168.0, text_letter_spacing),
            case("text-wrapping", 264.0, 464.0, text_wrapping),
            case("text-ellipsis", 264.0, 172.0, text_ellipsis),
            case("text-numbers", 184.0, 204.0, text_numbers),
        ],
    }
}

/// A case on a surface that holds its stage and a strip of bare surface under it.
fn case(name: &'static str, width: f32, height: f32, build: fn(&mut Scene)) -> Case {
    Case {
        name,
        viewport: Size { width, height },
        build,
        moment: Moment::AtRest,
    }
}

fn stage(scene: &mut Scene) -> ElementId {
    scene.add(scene.root(), &["stage"])
}

fn boxes(scene: &mut Scene) {
    let stage = stage(scene);
    let swatches = scene.add(stage, &["swatches"]);
    scene.add(swatches, &["swatch"]);
    scene.add(swatches, &["swatch", "accent"]);
    scene.add(swatches, &["swatch", "pill"]);
    scene.add(swatches, &["swatch", "tab"]);
    scene.add(swatches, &["swatch", "thick"]);
    scene.add(swatches, &["swatch", "sunken"]);
    scene.add(swatches, &["swatch", "marked"]);
    scene.add(swatches, &["swatch", "ring"]);
    let hovered = scene.add(swatches, &["swatch"]);
    scene.set_state(hovered, States::HOVER);
    let pressed = scene.add(swatches, &["swatch", "accent"]);
    scene.set_state(pressed, States::HOVER.union(States::ACTIVE));
    let disabled = scene.add(swatches, &["swatch", "pill"]);
    scene.set_state(disabled, States::DISABLED);
}

fn shadows(scene: &mut Scene) {
    let stage = stage(scene);
    let elevations = scene.add(stage, &["elevations"]);
    scene.add(elevations, &["card", "raised"]);
    scene.add(elevations, &["card", "float"]);
    scene.add(elevations, &["card", "modal"]);
}

fn gradients(scene: &mut Scene) {
    let stage = stage(scene);
    let gradients = scene.add(stage, &["gradients"]);
    scene.add(gradients, &["gradient", "across"]);
    scene.add(gradients, &["gradient", "diagonal"]);
    scene.add(gradients, &["gradient", "fading"]);
    scene.add(gradients, &["gradient", "spot"]);
}

fn overlap(scene: &mut Scene) {
    let stage = stage(scene);
    let stack = scene.add(stage, &["stack"]);
    scene.add(stack, &["plate", "focused"]);
    scene.add(stack, &["plate"]);
    scene.add(stack, &["plate"]);
    scene.add(stack, &["badge"]);
    let porthole = scene.add(stage, &["porthole"]);
    scene.add(porthole, &["flood"]);
}

fn flex(scene: &mut Scene) {
    let stage = stage(scene);
    let rows = scene.add(stage, &["rows"]);

    let plain = scene.add(rows, &["row"]);
    scene.add(plain, &["chip"]);
    scene.add(plain, &["chip", "tall"]);
    scene.add(plain, &["chip", "growing"]);
    scene.add(plain, &["chip"]);

    let centered = scene.add(rows, &["row", "centered"]);
    scene.add(centered, &["chip"]);
    scene.add(centered, &["chip", "tall"]);
    scene.add(centered, &["chip"]);

    let spread = scene.add(rows, &["row", "spread"]);
    scene.add(spread, &["chip"]);
    scene.add(spread, &["chip", "tall"]);
    scene.add(spread, &["chip"]);

    let wrapping = scene.add(rows, &["row", "wrapping"]);
    scene.add(wrapping, &["chip", "wide"]);
    scene.add(wrapping, &["chip", "wide"]);
    scene.add(wrapping, &["chip", "half"]);
    scene.add(wrapping, &["chip", "pushed"]);

    let shrinking = scene.add(rows, &["row"]);
    scene.add(shrinking, &["chip", "rigid"]);
    scene.add(shrinking, &["chip", "wide"]);
    scene.add(shrinking, &["chip", "wide"]);
    scene.add(shrinking, &["chip", "wide"]);
}

fn grid(scene: &mut Scene) {
    let stage = stage(scene);
    let mixed = scene.add(stage, &["grid"]);
    scene.add(mixed, &["cell"]);
    scene.add(mixed, &["cell", "across"]);
    scene.add(mixed, &["cell", "down"]);
    scene.add(mixed, &["cell"]);
    scene.add(mixed, &["cell", "inset"]);
    scene.add(mixed, &["cell", "last"]);

    let even = scene.add(stage, &["grid", "even"]);
    for _ in 0..6 {
        scene.add(even, &["cell"]);
    }
}

fn text_sizes(scene: &mut Scene) {
    let stage = stage(scene);
    let lines = scene.add(stage, &["lines"]);
    scene.add_text(lines, &["size-xs"], PANGRAM);
    scene.add_text(lines, &["size-sm"], PANGRAM);
    scene.add_text(lines, &[], PANGRAM);
    scene.add_text(lines, &["size-lg"], PANGRAM);
    scene.add_text(lines, &["size-xl"], PANGRAM);
    scene.add_text(lines, &["size-2xl"], "The quick brown fox jumps");
}

fn text_weights(scene: &mut Scene) {
    let stage = stage(scene);
    let lines = scene.add(stage, &["lines", "size-lg"]);
    scene.add_text(lines, &[], PANGRAM);
    scene.add_text(lines, &["weight-medium"], PANGRAM);
    scene.add_text(lines, &["weight-semibold"], PANGRAM);
    scene.add_text(lines, &["italic", "muted"], PANGRAM);
}

fn text_leading(scene: &mut Scene) {
    let stage = stage(scene);
    let lines = scene.add(stage, &["lines"]);
    for leading in ["leading-tight", "leading-normal"] {
        let column = scene.add(lines, &["column", leading]);
        scene.add_text(column, &[], PARAGRAPH);
    }
    let column = scene.add(lines, &["column"]);
    scene.add_text(column, &[], PARAGRAPH);
}

fn text_letter_spacing(scene: &mut Scene) {
    let stage = stage(scene);
    let lines = scene.add(stage, &["lines"]);
    scene.add_text(lines, &["tracking-tight"], PANGRAM);
    scene.add_text(lines, &[], PANGRAM);
    scene.add_text(lines, &["tracking-wide"], PANGRAM);
    scene.add_text(lines, &["tracking-wider"], PANGRAM);
}

fn text_wrapping(scene: &mut Scene) {
    let stage = stage(scene);
    let lines = scene.add(stage, &["lines"]);
    let wrapped = scene.add(lines, &["column"]);
    scene.add_text(wrapped, &[], PARAGRAPH);
    let centered = scene.add(lines, &["column", "middle"]);
    scene.add_text(centered, &[], PARAGRAPH);
    let right = scene.add(lines, &["column", "end", "muted"]);
    scene.add_text(right, &[], PARAGRAPH);
    let unbroken = scene.add(lines, &["column", "unbroken"]);
    scene.add_text(unbroken, &[], "One line, however narrow the box is");
    let long_word = scene.add(lines, &["column"]);
    scene.add_text(long_word, &[], "An uninterruptedlylongwordthatcannotbreak overflows");
}

fn text_ellipsis(scene: &mut Scene) {
    let stage = stage(scene);
    let lines = scene.add(stage, &["lines"]);
    let cut = scene.add(lines, &["column", "truncated"]);
    scene.add_text(cut, &[], PARAGRAPH);
    let fitting = scene.add(lines, &["column", "truncated"]);
    scene.add_text(fitting, &[], "A line that fits");
    let large = scene.add(lines, &["column", "truncated", "size-lg", "accented"]);
    scene.add_text(large, &[], PANGRAM);
}

fn text_numbers(scene: &mut Scene) {
    let stage = stage(scene);
    let lines = scene.add(stage, &["lines"]);
    for figures in [&["column", "figures"][..], &["column", "figures", "tabular"][..]] {
        let column = scene.add(lines, figures);
        scene.add_text(column, &[], "1,111,111.11");
        scene.add_text(column, &[], "9,876,543.20");
        scene.add_text(column, &[], "40,404.00");
    }
}
