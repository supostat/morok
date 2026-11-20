use morok_components::view::Mount;
use morok_components::{Combobox, Select};
use morok_core::{ElementId, Size};
use morok_css::SourceFile;
use morok_reactive::Signal;
use morok_style::States;

use crate::case::Case;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;
use crate::suites::{component_stylesheet, kit_tokens};

pub(crate) const NAME: &str = "select-combobox";

const COMPONENT_STYLESHEET_PATHS: [&str; 5] = [
    "crates/components/src/combobox.css",
    "crates/components/src/option_list.css",
    "crates/components/src/scroll_view.css",
    "crates/components/src/select.css",
    "crates/components/src/text_field.css",
];
const STYLESHEET_PATH: &str = "crates/teststand/suites/select-combobox.css";
const STYLESHEET: &str = include_str!("../../suites/select-combobox.css");

/// The states the tree gives a control the pointer presses.
const PRESSED: States = States::HOVER.union(States::ACTIVE);
/// The states the tree gives an element the keyboard focused.
const KEYBOARD_FOCUS: States = States::FOCUS.union(States::FOCUS_VISIBLE);

const SORTINGS: [&str; 4] = ["By name", "By size", "By kind", "By date modified"];
const FOLDERS: [&str; 5] = ["Applications", "Desktop", "Documents", "Downloads", "Movies"];

/// `Select` and `Combobox` closed, in every state, and open: the control with its list under
/// it, the row of the held option marked and a row highlighted.
pub(crate) fn suite() -> Suite {
    Suite {
        name: NAME,
        tokens: kit_tokens(),
        component_stylesheets: COMPONENT_STYLESHEET_PATHS
            .iter()
            .map(|path| component_stylesheet(path))
            .collect(),
        stylesheets: vec![SourceFile::new(STYLESHEET_PATH, STYLESHEET)],
        cases: vec![
            case("select-states", 716.0, 100.0, select_states),
            case("select-open", 284.0, 210.0, select_open),
            case("combobox-states", 716.0, 100.0, combobox_states),
            case("combobox-open", 284.0, 158.0, combobox_open),
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

fn grid(scene: &mut Scene) -> ElementId {
    let stage = scene.add(scene.root(), &["dropdowns-stage"]);
    scene.add(stage, &["dropdowns-grid"])
}

fn sorting_select_of(chosen: Signal<usize>) -> Select<usize> {
    SORTINGS
        .iter()
        .enumerate()
        .fold(Select::new(chosen), |select, (option, name)| {
            select.option(option, *name)
        })
}

/// Selects at rest, under the pointer, pressed, with the focus the keyboard gave and disabled.
fn select_states(scene: &mut Scene) {
    let grid = grid(scene);
    let select = |is_disabled: bool| {
        move |mount: &mut Mount<'_>| {
            let chosen = mount.runtime().signal(1);
            sorting_select_of(chosen).disabled(is_disabled).mount(mount)
        }
    };
    scene.mount(grid, select(false));
    let pointed = scene.mount(grid, select(false));
    scene.set_state(pointed, States::HOVER);
    let pressed = scene.mount(grid, select(false));
    scene.set_state(pressed, PRESSED);
    let focused = scene.mount(grid, select(false));
    scene.set_state(focused, KEYBOARD_FOCUS);
    scene.mount(grid, select(true));
}

/// A select over its list: the row of the chosen option carries the check mark and the row
/// after it the highlight.
fn select_open(scene: &mut Scene) {
    let stage = scene.add(scene.root(), &["dropdown-open"]);
    scene.mount(stage, |mount| {
        let chosen = mount.runtime().signal(1);
        sorting_select_of(chosen).mount(mount)
    });
    scene.mount(stage, |mount| {
        let chosen = mount.runtime().signal(1);
        sorting_select_of(chosen).mount_panel(mount)
    });
    highlight_row(scene, 2);
}

fn folder_combobox(mount: &mut Mount<'_>, text: &str) -> Combobox {
    let folder = mount.runtime().signal(String::from(text));
    FOLDERS
        .iter()
        .fold(Combobox::new(folder), |combobox, name| combobox.option(*name))
        .placeholder("Folder")
}

/// Combo boxes with their placeholder, filled, focused with the caret, invalid and disabled.
fn combobox_states(scene: &mut Scene) {
    let grid = grid(scene);
    scene.mount(grid, |mount| folder_combobox(mount, "").mount(mount));
    scene.mount(grid, |mount| folder_combobox(mount, "Documents").mount(mount));
    let focused = scene.mount(grid, |mount| folder_combobox(mount, "Down").mount(mount));
    focus_line(scene, focused, 4);
    scene.mount(grid, |mount| folder_combobox(mount, "Trash").invalid(true).mount(mount));
    scene.mount(grid, |mount| {
        folder_combobox(mount, "Movies").disabled(true).mount(mount)
    });
}

/// A combo box the user typed into over the list of the options the text leaves, the first
/// of them highlighted.
fn combobox_open(scene: &mut Scene) {
    let stage = scene.add(scene.root(), &["dropdown-open"]);
    let field = scene.mount(stage, |mount| folder_combobox(mount, "do").mount(mount));
    focus_line(scene, field, 2);
    scene.mount(stage, |mount| folder_combobox(mount, "do").mount_panel(mount));
    highlight_row(scene, 0);
}

/// Shows the field as the tree shows it once the pointer focused its line, with the caret at
/// the byte.
fn focus_line(scene: &mut Scene, field: ElementId, caret: usize) {
    let input = scene.tree().children(field)[0];
    scene.set_state(input, States::FOCUS);
    scene.set_state(field, States::FOCUS_WITHIN);
    scene.select_text(input, caret, caret);
}

/// Gives the row of the list of the scene the highlight the pointer or an arrow key gives it.
fn highlight_row(scene: &mut Scene, row: usize) {
    let highlighted = scene.elements_with_class("option")[row];
    scene.add_class(highlighted, "highlighted");
}
