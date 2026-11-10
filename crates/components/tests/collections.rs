mod support;

use std::borrow::Cow;

use morok_components::view::Mount;
use morok_components::{List, ListSelection, Tabs, Tree, TreeItem};
use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{ElementId, Key, Modifiers};
use morok_reactive::Signal;
use morok_style::{Declaration, LengthPercentage, LengthPercentageAuto, Pixels};
use support::Stage;

fn pixels(length: f32) -> LengthPercentageAuto {
    LengthPercentageAuto::LengthPercentage(LengthPercentage {
        pixels: Pixels(length),
        fraction: None,
        tokens: Cow::Borrowed(&[]),
    })
}

/// Gives a view of the kit 200 by 140 pixels.
fn sized(mount: &mut Mount<'_>, view: ElementId) -> ElementId {
    mount
        .build(view)
        .inline(Declaration::Width(pixels(200.0)))
        .inline(Declaration::Height(pixels(140.0)));
    view
}

/// The row elements of a view of rows, by the slot.
fn rows_of(stage: &Stage, view: ElementId) -> Vec<ElementId> {
    let viewport = stage.children(view)[0];
    let slots = stage.children(stage.children(viewport)[0]);
    slots.into_iter().map(|slot| stage.children(slot)[0]).collect()
}

fn list(selection: ListSelection) -> (Stage, ElementId, Signal<Vec<usize>>) {
    let mut stage = Stage::new();
    let selected = stage.runtime.signal(Vec::new());
    let view = stage.mount(move |mount| {
        let view = List::new(6, |index| format!("File {index}"), selected)
            .selection(selection)
            .label("Files")
            .mount(mount);
        sized(mount, view)
    });
    stage.frame();
    (stage, view, selected)
}

#[test]
fn a_click_and_the_arrow_keys_select_one_row_of_a_list() {
    let (mut stage, view, selected) = list(ListSelection::Single);
    let rows = rows_of(&stage, view);
    let viewport = stage.children(view)[0];

    stage.click(rows[2]);
    assert_eq!(stage.runtime.get(selected), [2]);
    assert!(stage.focus(viewport));
    assert!(stage.press_key(Key::ArrowDown, Modifiers::NONE));
    assert!(stage.press_key(Key::ArrowDown, Modifiers::NONE));
    assert_eq!(stage.runtime.get(selected), [4]);
    assert!(stage.press_key(Key::Home, Modifiers::NONE));
    assert_eq!(stage.runtime.get(selected), [0]);

    stage.frame();
    let node = stage.accessible_node(rows[0]);
    assert_eq!(node.role(), AccessibleRole::ListBoxOption);
    assert_eq!((node.label(), node.is_selected()), (Some("File 0"), Some(true)));
    assert_eq!((node.position_in_set(), node.size_of_set()), (Some(1), Some(6)));
    assert_eq!(stage.accessible_node(viewport).role(), AccessibleRole::ListBox);
}

#[test]
fn shift_and_space_select_more_rows_of_a_list_that_takes_many() {
    let (mut stage, view, selected) = list(ListSelection::Multiple);
    let viewport = stage.children(view)[0];
    stage.click(rows_of(&stage, view)[1]);
    assert!(stage.focus(viewport));

    assert!(stage.press_key(Key::ArrowDown, Modifiers::SHIFT));
    assert!(stage.press_key(Key::ArrowDown, Modifiers::SHIFT));
    assert_eq!(stage.runtime.get(selected), [1, 2, 3]);
    assert!(stage.press_key(Key::Space, Modifiers::NONE));
    assert_eq!(
        stage.runtime.get(selected),
        [1, 2],
        "Space takes the row the keyboard is on away"
    );
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
            children: vec![leaf("morok"), leaf("domovoi")],
        },
        leaf("Downloads"),
    ]
}

#[test]
fn the_arrow_keys_open_close_and_walk_a_tree() {
    let mut stage = Stage::new();
    let expanded = stage.runtime.signal(Vec::new());
    let selected = stage.runtime.signal(None);
    let view = stage.mount(move |mount| {
        let view = Tree::new(folders(), expanded, selected).label("Folders").mount(mount);
        sized(mount, view)
    });
    stage.frame();
    let viewport = stage.children(view)[0];
    assert_eq!(rows_of(&stage, view).len(), 2, "closed, the tree shows its top level");
    assert!(stage.focus(viewport));

    assert!(stage.press_key(Key::ArrowRight, Modifiers::NONE));
    assert_eq!(stage.runtime.get(expanded), ["Projects"]);
    stage.frame();
    assert_eq!(rows_of(&stage, view).len(), 4);
    assert!(stage.press_key(Key::ArrowRight, Modifiers::NONE));
    assert_eq!(stage.runtime.get(selected), Some("morok"), "an open item is gone into");
    stage.frame();
    let rows = rows_of(&stage, view);
    let node = stage.accessible_node(rows[1]);
    assert_eq!(node.role(), AccessibleRole::TreeItem);
    assert_eq!((node.level(), node.is_selected()), (Some(2), Some(true)));
    assert_eq!(stage.accessible_node(rows[0]).is_expanded(), Some(true));

    assert!(stage.press_key(Key::ArrowLeft, Modifiers::NONE));
    assert_eq!(
        stage.runtime.get(selected),
        Some("Projects"),
        "a closed item goes out to its parent"
    );
    assert!(stage.press_key(Key::ArrowLeft, Modifiers::NONE));
    assert!(stage.runtime.get(expanded).is_empty());
    stage.frame();
    assert_eq!(rows_of(&stage, view).len(), 2);
}

#[test]
fn a_click_and_the_arrow_keys_select_a_tab() {
    let mut stage = Stage::new();
    let shown = stage.runtime.signal("general");
    let strip = stage.mount(move |mount| {
        Tabs::new(shown)
            .tab("general", "General")
            .tab("storage", "Storage")
            .tab("network", "Network")
            .label("Settings")
            .mount(mount)
    });
    stage.frame();
    let tabs = stage.children(strip);

    stage.click(tabs[2]);
    assert_eq!(stage.runtime.get(shown), "network");
    assert!(stage.focus(tabs[2]));
    assert!(stage.press_key(Key::ArrowLeft, Modifiers::NONE));
    assert_eq!(stage.runtime.get(shown), "storage");

    stage.frame();
    assert_eq!(stage.accessible_node(strip).role(), AccessibleRole::TabList);
    let tab = stage.accessible_node(tabs[1]);
    assert_eq!((tab.role(), tab.is_selected()), (AccessibleRole::Tab, Some(true)));
    assert_eq!(stage.tree().focused(), Some(tabs[1]), "focus follows the selected tab");
}
