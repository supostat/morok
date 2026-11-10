use morok_core::{ElementId, Size};
use morok_css::SourceFile;

use crate::case::Case;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;

pub(crate) const NAME: &str = "effects";

const STYLESHEET_PATH: &str = "crates/teststand/suites/effects.css";
const STYLESHEET: &str = include_str!("../../suites/effects.css");

/// Transforms, group opacity, filters, blend modes and a backdrop filter.
pub(crate) fn suite() -> Suite {
    Suite {
        name: NAME,
        stylesheets: vec![SourceFile::new(STYLESHEET_PATH, STYLESHEET)],
        cases: vec![
            case("transforms", 640.0, 160.0, transforms),
            case("opacity", 416.0, 152.0, opacity),
            case("filters", 480.0, 300.0, filters),
            case("blend-modes", 520.0, 136.0, blend_modes),
            case("backdrop", 352.0, 192.0, backdrop),
        ],
    }
}

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

fn transforms(scene: &mut Scene) {
    let stage = stage(scene);
    let tiles = scene.add(stage, &["tiles"]);
    scene.add(tiles, &["tile"]);
    scene.add(tiles, &["tile", "moved"]);
    scene.add(tiles, &["tile", "grown"]);
    scene.add(tiles, &["tile", "squashed"]);
    scene.add(tiles, &["tile", "turned"]);
    scene.add(tiles, &["tile", "hinged"]);
    scene.add(tiles, &["tile", "carried"]);
}

fn opacity(scene: &mut Scene) {
    let stage = stage(scene);
    let sheets = scene.add(stage, &["sheets"]);
    let plain = scene.add(sheets, &["sheet"]);
    scene.add(plain, &["badge"]);
    let faded = scene.add(sheets, &["sheet", "faded"]);
    scene.add(faded, &["badge"]);
    let ghost = scene.add(sheets, &["sheet", "ghost"]);
    scene.add(ghost, &["badge"]);
}

fn filters(scene: &mut Scene) {
    let stage = stage(scene);
    let cards = scene.add(stage, &["cards"]);
    for variant in [
        "gray",
        "half-gray",
        "bright",
        "dim",
        "contrasty",
        "flat",
        "vivid",
        "pale",
        "thin",
        "soft",
        "graded",
    ] {
        scene.add(cards, &["card", variant]);
    }
    scene.add(cards, &["card"]);
}

fn blend_modes(scene: &mut Scene) {
    let stage = stage(scene);
    let backdrops = scene.add(stage, &["backdrops"]);
    for mode in ["multiply", "screen", "overlay", "darken", "lighten"] {
        let field = scene.add(backdrops, &["field"]);
        scene.add(field, &["disc", mode]);
    }
}

fn backdrop(scene: &mut Scene) {
    let stage = stage(scene);
    let glass_scene = scene.add(stage, &["scene"]);
    scene.add(glass_scene, &["half", "left"]);
    scene.add(glass_scene, &["half", "right"]);
    scene.add(glass_scene, &["glass"]);
}
