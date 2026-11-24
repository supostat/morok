mod support;

use std::borrow::Cow;
use std::fmt::Write;

use morok_components::view::Mount;
use morok_components::{List, RowSelection, Tabs, Tree, TreeItem};
use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{ElementId, Key, Modifiers, Point};
use morok_reactive::Signal;
use morok_style::{Declaration, LengthPercentage, LengthPercentageAuto, Pixels, States};
use support::Stage;

/// How high a row of a tree is.
const TREE_ROW_HEIGHT: f32 = 28.0;

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

fn list(selection: RowSelection) -> (Stage, ElementId, Signal<Vec<usize>>) {
    let mut stage = Stage::new();
    let selected = stage.runtime.signal(Vec::new());
    let view = stage.mount(move |mount| {
        let view = List::new(
            6,
            |index, label| write!(label, "File {index}").expect("a string takes what is written to it"),
            selected,
        )
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
    let (mut stage, view, selected) = list(RowSelection::Single);
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
    let (mut stage, view, selected) = list(RowSelection::Multiple);
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

/// Forty folders at the top level of a tree, named "Folder 00" to "Folder 39": more items than
/// the tree keeps row slots for.
fn forty_folders() -> Vec<TreeItem<String>> {
    (0..40)
        .map(|index| TreeItem {
            key: format!("folder-{index:02}"),
            label: format!("Folder {index:02}"),
            children: Vec::new(),
        })
        .collect()
}

/// A tree of `roots` with the item of the key `selected` selected, on a stage that ran its
/// first frame.
fn tree_of<ItemKey: Clone + PartialEq + 'static>(
    roots: Vec<TreeItem<ItemKey>>,
    selected: Option<ItemKey>,
) -> (Stage, ElementId, Signal<Vec<ItemKey>>) {
    let mut stage = Stage::new();
    let expanded = stage.runtime.signal(Vec::new());
    let selected = stage.runtime.signal(selected);
    let view = stage.mount(move |mount| {
        let view = Tree::new(roots, expanded, selected).label("Folders").mount(mount);
        sized(mount, view)
    });
    stage.frame();
    (stage, view, expanded)
}

/// The label a row of a tree shows: the text in the room past its chevron.
fn tree_label(stage: &Stage, row: ElementId) -> String {
    let [_chevron, room] = stage.tree().children(row) else {
        panic!("a row of a tree holds its chevron and the room of its label");
    };
    let [text] = stage.tree().children(*room) else {
        panic!("the room of a label holds its text");
    };
    stage.tree().text(*text).to_owned()
}

/// The labels of the rows of a tree its selection marks.
fn marked_in_tree(stage: &Stage, view: ElementId) -> Vec<String> {
    rows_of(stage, view)
        .into_iter()
        .filter(|row| stage.tree().states(*row).contains(States::CHECKED))
        .map(|row| tree_label(stage, row))
        .collect()
}

/// The wheel scrolls the viewport of a view of rows by `rows` rows of a tree, and a frame runs.
fn scroll_tree_by(stage: &mut Stage, view: ElementId, rows: f32) {
    let viewport = stage.children(view)[0];
    let position = stage.middle_on(stage.surface, viewport);
    stage.wheel_on(
        stage.surface,
        position,
        Point {
            x: 0.0,
            y: rows * TREE_ROW_HEIGHT,
        },
    );
    stage.frame();
}

#[test]
fn a_scroll_of_one_row_binds_one_slot_of_a_tree_to_the_text_and_the_name_of_the_item_that_came_into_view() {
    let (mut stage, view, _) = tree_of(forty_folders(), None);
    let rows = rows_of(&stage, view);
    let before: Vec<String> = rows.iter().map(|row| tree_label(&stage, *row)).collect();

    scroll_tree_by(&mut stage, view, 1.0);

    assert_eq!(rows_of(&stage, view), rows, "every slot keeps its row element");
    let after: Vec<String> = rows.iter().map(|row| tree_label(&stage, *row)).collect();
    let rebound: Vec<usize> = (0..rows.len()).filter(|slot| before[*slot] != after[*slot]).collect();
    assert_eq!(
        rebound.len(),
        1,
        "one slot shows another item: {before:?} then {after:?}"
    );
    assert_eq!(after[rebound[0]], "Folder 32");
    assert_eq!(
        stage.accessible_node(rows[rebound[0]]).label(),
        Some("Folder 32"),
        "an assistive technology hears the item the slot shows now"
    );
}

#[test]
fn the_mark_of_the_selected_item_follows_the_item_and_leaves_the_slot_it_moves_out_of() {
    let (mut stage, view, _) = tree_of(forty_folders(), Some(String::from("folder-02")));
    assert_eq!(marked_in_tree(&stage, view), ["Folder 02"]);

    scroll_tree_by(&mut stage, view, 3.0);
    assert!(
        marked_in_tree(&stage, view).is_empty(),
        "the slot of the selected item shows another item, unmarked: {:?}",
        rows_of(&stage, view)
            .iter()
            .map(|row| tree_label(&stage, *row))
            .collect::<Vec<_>>()
    );
    scroll_tree_by(&mut stage, view, -3.0);
    assert_eq!(marked_in_tree(&stage, view), ["Folder 02"]);

    let (mut stage, view, expanded) = tree_of(folders(), Some("Downloads"));
    assert_eq!(marked_in_tree(&stage, view), ["Downloads"]);
    let marked_row = rows_of(&stage, view)[1];
    stage.runtime.set(expanded, vec!["Projects"]);
    stage.frame();
    assert_eq!(
        marked_in_tree(&stage, view),
        ["Downloads"],
        "opening the folder above moves the selected item down a row and the mark with it"
    );
    assert!(!stage.tree().states(marked_row).contains(States::CHECKED));
    assert_eq!(tree_label(&stage, marked_row), "morok");
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
