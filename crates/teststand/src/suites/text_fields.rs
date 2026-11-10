use morok_components::view::Mount;
use morok_components::{SearchField, TextArea, TextField};
use morok_core::{ElementId, Size};
use morok_css::SourceFile;
use morok_style::States;

use crate::case::Case;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;
use crate::suites::component_stylesheet;

pub(crate) const NAME: &str = "text-fields";

const COMPONENT_STYLESHEET_PATHS: [&str; 3] = [
    "crates/components/src/search_field.css",
    "crates/components/src/text_area.css",
    "crates/components/src/text_field.css",
];
const STYLESHEET_PATH: &str = "crates/teststand/suites/text-fields.css";
const STYLESHEET: &str = include_str!("../../suites/text-fields.css");

/// The width of a stage of three columns of fields.
const STAGE_WIDTH: f32 = 716.0;
/// A stage of two rows of fields one line high, and one of two rows of text areas.
const LINE_STAGE_HEIGHT: f32 = 100.0;
const AREA_STAGE_HEIGHT: f32 = 208.0;

const NOTE: &str = "Delete Xcode caches older than a month and empty the trash";

/// `TextField`, `SearchField` and `TextArea` in every state: with their placeholder, filled,
/// focused with the caret, focused with a selection, invalid and disabled.
pub(crate) fn suite() -> Suite {
    Suite {
        name: NAME,
        component_stylesheets: COMPONENT_STYLESHEET_PATHS
            .iter()
            .map(|path| component_stylesheet(path))
            .collect(),
        stylesheets: vec![SourceFile::new(STYLESHEET_PATH, STYLESHEET)],
        cases: vec![
            case("text-field-states", LINE_STAGE_HEIGHT, text_field_states),
            case("search-field-states", LINE_STAGE_HEIGHT, search_field_states),
            case("text-area-states", AREA_STAGE_HEIGHT, text_area_states),
        ],
    }
}

fn case(name: &'static str, height: f32, build: fn(&mut Scene)) -> Case {
    Case {
        name,
        viewport: Size {
            width: STAGE_WIDTH,
            height,
        },
        build,
        moment: Moment::AtRest,
    }
}

/// How a field of a case is shown beyond what it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shown {
    AtRest,
    /// Focused, with the caret at the byte.
    Caret(usize),
    /// Focused, with the bytes selected.
    Selection(usize, usize),
}

fn text_field_states(scene: &mut Scene) {
    let grid = grid(scene);
    let field = |value: &str, build: fn(TextField) -> TextField| {
        let value = String::from(value);
        move |mount: &mut Mount<'_>| {
            let value = mount.runtime().signal(value);
            build(TextField::new(value).placeholder("Folder name")).mount(mount)
        }
    };
    field_in(scene, grid, Shown::AtRest, field("", |field| field));
    field_in(scene, grid, Shown::AtRest, field("Report for November", |field| field));
    field_in(scene, grid, Shown::Caret(7), field("Cleanup", |field| field));
    field_in(
        scene,
        grid,
        Shown::Selection(11, 19),
        field("Report for November 2026", |field| field),
    );
    field_in(
        scene,
        grid,
        Shown::AtRest,
        field("report*.pdf?", |field| field.invalid(true)),
    );
    field_in(
        scene,
        grid,
        Shown::AtRest,
        field("Archive", |field| field.disabled(true)),
    );
}

fn search_field_states(scene: &mut Scene) {
    let grid = grid(scene);
    let field = |value: &str, build: fn(SearchField) -> SearchField| {
        let value = String::from(value);
        move |mount: &mut Mount<'_>| {
            let value = mount.runtime().signal(value);
            build(SearchField::new(value).placeholder("Search")).mount(mount)
        }
    };
    field_in(scene, grid, Shown::AtRest, field("", |field| field));
    field_in(scene, grid, Shown::Caret(12), field("Xcode caches", |field| field));
    field_in(
        scene,
        grid,
        Shown::Selection(6, 12),
        field("Xcode caches", |field| field),
    );
    field_in(
        scene,
        grid,
        Shown::AtRest,
        field("Archive", |field| field.disabled(true)),
    );
}

fn text_area_states(scene: &mut Scene) {
    let grid = grid(scene);
    let area = |value: &str, build: fn(TextArea) -> TextArea| {
        let value = String::from(value);
        move |mount: &mut Mount<'_>| {
            let value = mount.runtime().signal(value);
            build(TextArea::new(value).placeholder("Note about this cleanup")).mount(mount)
        }
    };
    field_in(scene, grid, Shown::AtRest, area("", |area| area));
    field_in(scene, grid, Shown::Caret(NOTE.len()), area(NOTE, |area| area));
    field_in(scene, grid, Shown::Selection(13, 19), area(NOTE, |area| area));
    field_in(scene, grid, Shown::AtRest, area(NOTE, |area| area.invalid(true)));
    field_in(scene, grid, Shown::AtRest, area(NOTE, |area| area.disabled(true)));
    field_in(scene, grid, Shown::AtRest, area("Keep the downloads", |area| area));
}

fn grid(scene: &mut Scene) -> ElementId {
    let stage = scene.add(scene.root(), &["fields-stage"]);
    scene.add(stage, &["fields-grid"])
}

/// Mounts the field in the grid and shows it as the case asks: a focused field has the
/// states the tree gives it when the pointer focuses its line.
fn field_in(scene: &mut Scene, grid: ElementId, shown: Shown, field: impl FnOnce(&mut Mount<'_>) -> ElementId) {
    let mounted = scene.mount(grid, field);
    let input = *scene
        .tree()
        .children(mounted)
        .last()
        .expect("a field holds its edited text last");
    match shown {
        Shown::AtRest => {}
        Shown::Caret(offset) => {
            focus(scene, mounted, input);
            scene.select_text(input, offset, offset);
        }
        Shown::Selection(anchor, head) => {
            focus(scene, mounted, input);
            scene.select_text(input, anchor, head);
        }
    }
}

fn focus(scene: &mut Scene, field: ElementId, input: ElementId) {
    scene.set_state(input, States::FOCUS);
    scene.set_state(field, States::FOCUS_WITHIN);
}
