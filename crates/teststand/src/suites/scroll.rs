use morok_components::ScrollView;
use morok_core::{ElementId, Point, Size};
use morok_css::SourceFile;
use morok_style::States;

use crate::case::Case;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;
use crate::suites::component_stylesheet;

pub(crate) const NAME: &str = "scroll";

const COMPONENT_STYLESHEET_PATH: &str = "crates/components/src/scroll_view.css";
const STYLESHEET_PATH: &str = "crates/teststand/suites/scroll.css";
const STYLESHEET: &str = include_str!("../../suites/scroll.css");

/// Three views side by side, each 182 pixels wide with its border.
const STAGE_WIDTH: f32 = 620.0;
const STAGE_HEIGHT: f32 = 180.0;

/// `ScrollView` scrolled to fixed offsets: lists at the start, between rows and past the end,
/// sticky section headers held at the top and pushed out by the end of their section, wide
/// rows scrolled across. The bars are under the pointer, so their thumbs show.
pub(crate) fn suite() -> Suite {
    Suite {
        name: NAME,
        component_stylesheets: vec![component_stylesheet(COMPONENT_STYLESHEET_PATH)],
        stylesheets: vec![SourceFile::new(STYLESHEET_PATH, STYLESHEET)],
        cases: vec![
            case("list-offsets", list_offsets),
            case("sticky-sections", sticky_sections),
            case("wide-rows", wide_rows),
        ],
    }
}

fn case(name: &'static str, build: fn(&mut Scene)) -> Case {
    Case {
        name,
        viewport: Size {
            width: STAGE_WIDTH,
            height: STAGE_HEIGHT,
        },
        build,
        moment: Moment::AtRest,
    }
}

fn stage(scene: &mut Scene) -> ElementId {
    scene.add(scene.root(), &["scroll-stage"])
}

/// Mounts a view on the stage with its bars under the pointer and returns its viewport and
/// the element its content is built in.
fn view(scene: &mut Scene, stage: ElementId) -> (ElementId, ElementId) {
    let view = scene.mount(stage, |mount| {
        ScrollView::new(|mount| mount.element().id()).mount(mount)
    });
    let parts = scene.tree().children(view).to_vec();
    let [viewport, vertical_bar, horizontal_bar] = parts[..] else {
        unreachable!("a scroll view holds its viewport and two bars");
    };
    scene.set_state(vertical_bar, States::HOVER);
    scene.set_state(horizontal_bar, States::HOVER);
    let content = scene.tree().children(viewport)[0];
    (viewport, content)
}

/// Rows of text from `first` on, as many as `count`, of the given classes.
fn rows(scene: &mut Scene, parent: ElementId, first: usize, count: usize, classes: &[&str]) {
    for number in first..first + count {
        let row = scene.add(parent, classes);
        scene.add_text(row, &[], &format!("Row {number}"));
    }
}

fn list_offsets(scene: &mut Scene) {
    let stage = stage(scene);
    for offset in [0.0, 57.0, 1000.0] {
        let (viewport, content) = view(scene, stage);
        let list = scene.add(content, &["list"]);
        rows(scene, list, 1, 10, &["list-row"]);
        scene.scroll_to(viewport, Point { x: 0.0, y: offset });
    }
}

fn sticky_sections(scene: &mut Scene) {
    let stage = stage(scene);
    for offset in [40.0, 100.0, 140.0] {
        let (viewport, content) = view(scene, stage);
        for section in 1..=4 {
            let list = scene.add(content, &["list"]);
            let header = scene.add(list, &["section-header"]);
            scene.add_text(header, &[], &format!("Section {section}"));
            rows(scene, list, section * 10 + 1, 3, &["list-row"]);
        }
        scene.scroll_to(viewport, Point { x: 0.0, y: offset });
    }
}

fn wide_rows(scene: &mut Scene) {
    let stage = stage(scene);
    for offset in [
        Point { x: 0.0, y: 0.0 },
        Point { x: 120.0, y: 30.0 },
        Point { x: 400.0, y: 400.0 },
    ] {
        let (viewport, content) = view(scene, stage);
        let list = scene.add(content, &["list"]);
        rows(scene, list, 1, 8, &["list-row", "wide"]);
        scene.scroll_to(viewport, offset);
    }
}
