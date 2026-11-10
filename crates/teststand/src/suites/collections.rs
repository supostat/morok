use morok_components::view::Mount;
use morok_components::{List, ListSelection, Tabs, Tree, TreeItem};
use morok_core::{ElementId, Size};
use morok_css::SourceFile;
use morok_style::States;

use crate::case::Case;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;
use crate::suites::component_stylesheet;

pub(crate) const NAME: &str = "collections";

const COMPONENT_STYLESHEET_PATHS: [&str; 4] = [
    "crates/components/src/list.css",
    "crates/components/src/scroll_view.css",
    "crates/components/src/tabs.css",
    "crates/components/src/tree.css",
];
const STYLESHEET_PATH: &str = "crates/teststand/suites/collections.css";
const STYLESHEET: &str = include_str!("../../suites/collections.css");

/// The states the tree gives a row the pointer presses.
const PRESSED: States = States::HOVER.union(States::ACTIVE);
/// The states the tree gives an element the keyboard focused.
const KEYBOARD_FOCUS: States = States::FOCUS.union(States::FOCUS_VISIBLE);

/// `List`, `Tree` and `Tabs` in every state: rows selected, under the pointer and pressed,
/// views with the focus the keyboard gave, items in levels opened and closed, tabs selected,
/// under the pointer, pressed, focused and disabled.
pub(crate) fn suite() -> Suite {
    Suite {
        name: NAME,
        component_stylesheets: COMPONENT_STYLESHEET_PATHS
            .iter()
            .map(|path| component_stylesheet(path))
            .collect(),
        stylesheets: vec![SourceFile::new(STYLESHEET_PATH, STYLESHEET)],
        cases: vec![
            case("list-states", 470.0, 180.0, list_states),
            case("tree-states", 470.0, 180.0, tree_states),
            case("tabs-states", 360.0, 210.0, tabs_states),
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

/// The row elements of a view of rows, by the slot.
fn rows_of(scene: &Scene, view: ElementId) -> Vec<ElementId> {
    let tree = scene.tree();
    let viewport = tree.children(view)[0];
    let slots = tree.children(tree.children(viewport)[0]).to_vec();
    slots.into_iter().map(|slot| tree.children(slot)[0]).collect()
}

fn viewport_of(scene: &Scene, view: ElementId) -> ElementId {
    scene.tree().children(view)[0]
}

/// A list of one selected row, a row under the pointer and a pressed one; and a list that
/// takes many rows, two of them selected, with the focus the keyboard gave.
fn list_states(scene: &mut Scene) {
    let stage = scene.add(scene.root(), &["collections-stage"]);
    let list = |selected: Vec<usize>, selection: ListSelection| {
        move |mount: &mut Mount<'_>| {
            let selected = mount.runtime().signal(selected);
            List::new(12, |index| format!("Document {}", index + 1), selected)
                .selection(selection)
                .mount(mount)
        }
    };
    let pointed = scene.mount(stage, list(vec![1], ListSelection::Single));
    let rows = rows_of(scene, pointed);
    scene.set_state(rows[3], States::HOVER);
    scene.set_state(rows[4], PRESSED);
    let focused = scene.mount(stage, list(vec![0, 2], ListSelection::Multiple));
    scene.set_state(viewport_of(scene, focused), KEYBOARD_FOCUS);
}

fn folders() -> Vec<TreeItem<&'static str>> {
    let leaf = |key: &'static str| TreeItem {
        key,
        label: key.to_owned(),
        children: Vec::new(),
    };
    vec![
        TreeItem {
            key: "Projects",
            label: String::from("Projects"),
            children: vec![
                TreeItem {
                    key: "morok",
                    label: String::from("morok"),
                    children: vec![leaf("crates"), leaf("design")],
                },
                leaf("domovoi"),
            ],
        },
        leaf("Downloads"),
        leaf("Music"),
    ]
}

/// A tree open two levels deep with an item selected and one under the pointer; and the same
/// tree closed, with the focus the keyboard gave.
fn tree_states(scene: &mut Scene) {
    let stage = scene.add(scene.root(), &["collections-stage"]);
    let tree = |open: Vec<&'static str>, selected: Option<&'static str>| {
        move |mount: &mut Mount<'_>| {
            let expanded = mount.runtime().signal(open);
            let selected = mount.runtime().signal(selected);
            Tree::new(folders(), expanded, selected).mount(mount)
        }
    };
    let open = scene.mount(stage, tree(vec!["Projects", "morok"], Some("crates")));
    let rows = rows_of(scene, open);
    scene.set_state(rows[5], States::HOVER);
    let closed = scene.mount(stage, tree(Vec::new(), Some("Downloads")));
    scene.set_state(viewport_of(scene, closed), KEYBOARD_FOCUS);
}

/// Strips of three tabs: at rest; with a tab under the pointer and one pressed; with the focus
/// the keyboard gave on the selected tab; disabled.
fn tabs_states(scene: &mut Scene) {
    let stage = scene.add(scene.root(), &["tabs-stage"]);
    let strip = |selected: &'static str, is_disabled: bool| {
        move |mount: &mut Mount<'_>| {
            let selected = mount.runtime().signal(selected);
            Tabs::new(selected)
                .tab("general", "General")
                .tab("storage", "Storage")
                .tab("network", "Network")
                .disabled(is_disabled)
                .mount(mount)
        }
    };
    scene.mount(stage, strip("general", false));
    let pointed = scene.mount(stage, strip("general", false));
    let tabs = scene.tree().children(pointed).to_vec();
    scene.set_state(tabs[1], States::HOVER);
    scene.set_state(tabs[2], PRESSED);
    let focused = scene.mount(stage, strip("storage", false));
    let tabs = scene.tree().children(focused).to_vec();
    scene.set_state(tabs[1], KEYBOARD_FOCUS);
    scene.mount(stage, strip("network", true));
}
