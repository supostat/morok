mod support;

use std::time::Duration;

use morok_components::styles::CLASSES;
use morok_components::view::SurfaceId;
use morok_components::{Button, Combobox, Select};
use morok_core::accesskit::{Live, Role as AccessibleRole};
use morok_core::{
    ElementId, Key, Modifiers, OverlayAlignment, OverlayInput, OverlaySide, Point, PointerButton, PointerInputKind,
    Size,
};
use morok_reactive::Signal;
use morok_style::States;
use support::Stage;

/// How high a row of a dropdown list is.
const ROW_HEIGHT: f32 = 26.0;
/// The padding and the border of the panel of a dropdown list, above and below its rows.
const PANEL_EDGES: f32 = 10.0;

/// Clicks the element of the surface of the window and runs the frames that follow.
fn click(stage: &mut Stage, element: ElementId) {
    stage.click(element);
    stage.frame_all();
}

/// A key goes down and up on the surface of the window, which has the keyboard, and the
/// frames that follow run. Returns whether the press was used.
fn press(stage: &mut Stage, key: Key) -> bool {
    let was_used = stage.press_key(key, Modifiers::NONE);
    stage.frame_all();
    was_used
}

/// The keys that type the characters of `text` go down and up one after another.
fn press_characters(stage: &mut Stage, text: &str) {
    for character in text.chars() {
        let key = if character == ' ' {
            Key::Space
        } else {
            Key::Character(character)
        };
        assert!(press(stage, key), "a key that types into a name is used");
    }
}

/// The only overlay opened over the surface of the window.
fn overlay_over_window(stage: &Stage) -> SurfaceId {
    let overlays = stage.overlays_of(stage.surface);
    assert_eq!(overlays.len(), 1, "one overlay is open over the window");
    overlays[0]
}

/// What an overlay shows: the panel, inside the box that leaves room for its shadow.
fn panel_of(stage: &Stage, overlay: SurfaceId) -> ElementId {
    let tree = stage.tree_of(overlay);
    let [frame] = tree.children(tree.root()) else {
        panic!("an overlay surface holds one box");
    };
    let [panel] = tree.children(*frame) else {
        panic!("the box of an overlay surface holds the panel");
    };
    *panel
}

/// The viewport the rows of a dropdown list scroll in.
fn viewport_of(stage: &Stage, overlay: SurfaceId) -> ElementId {
    let tree = stage.tree_of(overlay);
    let [view] = tree.children(panel_of(stage, overlay)) else {
        panic!("the panel of a dropdown list holds the view of its rows");
    };
    tree.children(*view)[0]
}

/// The row elements of a dropdown list, by the slot.
fn rows_of(stage: &Stage, overlay: SurfaceId) -> Vec<ElementId> {
    let tree = stage.tree_of(overlay);
    let slots = tree.children(tree.children(viewport_of(stage, overlay))[0]);
    slots.iter().map(|slot| tree.children(*slot)[0]).collect()
}

/// The name a row of a dropdown list shows, after its check mark.
fn name_of(stage: &Stage, overlay: SurfaceId, row: ElementId) -> String {
    let tree = stage.tree_of(overlay);
    let [mark, name] = tree.children(row) else {
        panic!("a row holds its check mark and the name of its option");
    };
    assert_eq!(tree.classes(*mark), [CLASSES.option_mark]);
    tree.text(*name).to_owned()
}

fn names_of(stage: &Stage, overlay: SurfaceId) -> Vec<String> {
    rows_of(stage, overlay)
        .into_iter()
        .map(|row| name_of(stage, overlay, row))
        .collect()
}

/// The names of the highlighted rows of a dropdown list: at most one row is highlighted.
fn highlighted_in(stage: &Stage, overlay: SurfaceId) -> Vec<String> {
    rows_of(stage, overlay)
        .into_iter()
        .filter(|row| stage.tree_of(overlay).classes(*row).contains(&CLASSES.highlighted))
        .map(|row| name_of(stage, overlay, row))
        .collect()
}

/// The pointer moves to the middle of a row of a dropdown list, and the frames that follow run.
fn point_at(stage: &mut Stage, overlay: SurfaceId, row: ElementId) {
    let position = stage.middle_on(overlay, row);
    stage.pointer_on(overlay, PointerInputKind::Moved, position);
    stage.frame_all();
}

/// Clicks a row of a dropdown list, and the frames that follow run.
fn click_row(stage: &mut Stage, overlay: SurfaceId, row: ElementId) {
    let position = stage.middle_on(overlay, row);
    stage.click_on(overlay, PointerButton::Primary, position);
    stage.frame_all();
}

mod select {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Sorting {
        Name,
        Size,
        Kind,
        Modified,
    }

    const NAMES: [&str; 4] = ["By name", "By size", "By kind", "By date modified"];

    fn sorting_select(sorting: Signal<Sorting>) -> Select<Sorting> {
        Select::new(sorting)
            .option(Sorting::Name, NAMES[0])
            .option(Sorting::Size, NAMES[1])
            .option(Sorting::Kind, NAMES[2])
            .option(Sorting::Modified, NAMES[3])
            .label("Sort order")
    }

    /// A select that holds the sorting by size, under the root of the window.
    fn select() -> (Stage, ElementId, Signal<Sorting>) {
        let mut stage = Stage::new();
        let sorting = stage.runtime.signal(Sorting::Size);
        let select = stage.mount(move |mount| sorting_select(sorting).mount(mount));
        stage.frame_all();
        (stage, select, sorting)
    }

    #[test]
    fn a_select_shows_the_chosen_option_and_drops_its_list_down_under_itself() {
        let (mut stage, select, _) = select();
        let tree = stage.tree();
        assert_eq!(tree.classes(select), [CLASSES.select]);
        let [name, chevron] = tree.children(select) else {
            panic!("a select holds the name of the chosen option and a chevron");
        };
        assert_eq!(
            (tree.text(*name), tree.classes(*name)),
            ("By size", [CLASSES.select_name].as_slice())
        );
        assert_eq!(tree.classes(*chevron), [CLASSES.select_chevron]);
        let node = stage.accessible_node(select);
        assert_eq!(
            (node.role(), node.label(), node.value(), node.is_expanded()),
            (
                AccessibleRole::ComboBox,
                Some("Sort order"),
                Some("By size"),
                Some(false)
            )
        );
        assert!(
            stage.overlays_of(stage.surface).is_empty(),
            "a closed select has no surface"
        );

        click(&mut stage, select);

        let overlay = overlay_over_window(&stage);
        let geometry = stage
            .surfaces
            .overlay_geometry(overlay)
            .expect("the select has a box after the frame");
        let select_box = stage.tree().border_box(select);
        assert_eq!(geometry.anchor, select_box);
        assert_eq!(
            (geometry.placement.side, geometry.placement.alignment),
            (OverlaySide::Below, OverlayAlignment::Start)
        );
        assert_eq!(
            geometry.input,
            OverlayInput::Pointer,
            "the list takes the pointer and leaves the keyboard with the select"
        );
        assert_eq!(
            (geometry.panel.width, geometry.panel.height),
            (select_box.width, 4.0 * ROW_HEIGHT + PANEL_EDGES),
            "the list is as wide as the select and as high as its four rows"
        );
        assert_eq!(
            stage.tree_of(overlay).classes(panel_of(&stage, overlay)),
            [CLASSES.options]
        );
        assert_eq!(names_of(&stage, overlay), NAMES);
        assert_eq!(
            highlighted_in(&stage, overlay),
            ["By size"],
            "the list opens with the highlight on the chosen option"
        );
        let rows = rows_of(&stage, overlay);
        assert_eq!(stage.tree_of(overlay).classes(rows[0]), [CLASSES.option]);
        assert!(stage.tree_of(overlay).states(rows[1]).contains(States::CHECKED));
        assert!(!stage.tree_of(overlay).states(rows[0]).contains(States::CHECKED));

        let viewport = viewport_of(&stage, overlay);
        let list = stage.accessible_node_on(overlay, viewport);
        assert_eq!(
            (list.role(), list.label()),
            (AccessibleRole::ListBox, Some("Sort order"))
        );
        let chosen = stage.accessible_node_on(overlay, rows[1]);
        assert_eq!(
            (chosen.role(), chosen.label(), chosen.is_selected()),
            (AccessibleRole::ListBoxOption, Some("By size"), Some(true))
        );
        assert_eq!((chosen.position_in_set(), chosen.size_of_set()), (Some(2), Some(4)));
        assert_eq!(stage.accessible_node_on(overlay, rows[0]).is_selected(), Some(false));
        assert_eq!(stage.accessible_node(select).is_expanded(), Some(true));
    }

    #[test]
    fn the_pointer_highlights_the_row_it_moves_over_and_a_click_chooses_its_option() {
        let (mut stage, select, sorting) = select();
        click(&mut stage, select);
        let overlay = overlay_over_window(&stage);
        let rows = rows_of(&stage, overlay);

        point_at(&mut stage, overlay, rows[3]);
        assert_eq!(highlighted_in(&stage, overlay), ["By date modified"]);
        assert_eq!(stage.runtime.get(sorting), Sorting::Size, "a highlight chooses nothing");

        click_row(&mut stage, overlay, rows[3]);

        assert_eq!(stage.runtime.get(sorting), Sorting::Modified);
        assert!(
            stage.overlays_of(stage.surface).is_empty(),
            "choosing an option closes the list"
        );
        assert!(!stage.surfaces.contains(overlay));
        let name = stage.tree().children(select)[0];
        assert_eq!(stage.tree().text(name), "By date modified");
        assert_eq!(stage.accessible_node(select).value(), Some("By date modified"));
        assert_eq!(
            stage.tree().focused(),
            Some(select),
            "the keyboard stayed with the select"
        );
    }

    #[test]
    fn the_keys_of_the_select_open_its_list_move_the_highlight_and_choose() {
        let (mut stage, select, sorting) = select();
        assert!(stage.focus(select));

        assert!(press(&mut stage, Key::ArrowDown), "the arrow opens the list");
        let overlay = overlay_over_window(&stage);
        assert_eq!(highlighted_in(&stage, overlay), ["By size"]);

        for expected in ["By kind", "By date modified", "By date modified"] {
            assert!(press(&mut stage, Key::ArrowDown));
            assert_eq!(
                highlighted_in(&stage, overlay),
                [expected],
                "the highlight stops at the last row"
            );
        }
        assert!(press(&mut stage, Key::Home));
        assert_eq!(highlighted_in(&stage, overlay), ["By name"]);
        assert!(press(&mut stage, Key::ArrowUp));
        assert_eq!(highlighted_in(&stage, overlay), ["By name"], "and at the first");
        assert!(press(&mut stage, Key::End));
        assert!(press(&mut stage, Key::ArrowUp));
        assert_eq!(highlighted_in(&stage, overlay), ["By kind"]);
        assert_eq!(
            stage.runtime.get(sorting),
            Sorting::Size,
            "the arrows of an open list choose nothing"
        );
        assert_eq!(
            stage.tree_of(overlay).focused(),
            None,
            "the list never has the keyboard"
        );

        assert!(press(&mut stage, Key::Enter));

        assert_eq!(stage.runtime.get(sorting), Sorting::Kind);
        assert!(stage.overlays_of(stage.surface).is_empty());
        assert_eq!(stage.tree().focused(), Some(select));

        assert!(press(&mut stage, Key::Space), "Space opens the list as well");
        let overlay = overlay_over_window(&stage);
        assert_eq!(highlighted_in(&stage, overlay), ["By kind"]);
        assert!(press(&mut stage, Key::ArrowUp));
        assert!(press(&mut stage, Key::Escape));
        assert!(stage.overlays_of(stage.surface).is_empty());
        assert_eq!(
            stage.runtime.get(sorting),
            Sorting::Kind,
            "Escape closes the list and leaves the choice"
        );
        assert!(!press(&mut stage, Key::Escape), "Escape is nothing to a closed select");
    }

    #[test]
    fn tab_closes_an_open_list_on_its_way_to_the_next_stop() {
        let (mut stage, select, sorting) = select();
        let next = stage.mount(|mount| Button::new("Next").mount(mount));
        stage.frame_all();
        assert!(stage.focus(select));
        press(&mut stage, Key::ArrowDown);
        press(&mut stage, Key::ArrowDown);
        assert_eq!(stage.overlays_of(stage.surface).len(), 1);

        assert!(press(&mut stage, Key::Tab));

        assert!(stage.overlays_of(stage.surface).is_empty());
        assert_eq!(stage.tree().focused(), Some(next));
        assert_eq!(stage.runtime.get(sorting), Sorting::Size);
    }

    #[test]
    fn typing_the_start_of_a_name_chooses_its_option_while_the_list_is_closed() {
        let (mut stage, select, sorting) = select();
        assert!(stage.focus(select));

        press_characters(&mut stage, "by k");

        assert_eq!(
            stage.runtime.get(sorting),
            Sorting::Kind,
            "the characters typed one after another make up the start of one name"
        );
        assert!(
            stage.overlays_of(stage.surface).is_empty(),
            "the space inside a name does not press the select"
        );

        stage.frame_all_after(Duration::from_millis(1100));
        stage.frame_all();
        press_characters(&mut stage, "b");

        assert_eq!(
            stage.runtime.get(sorting),
            Sorting::Name,
            "after a pause a new name begins"
        );

        press_characters(&mut stage, "x");
        assert_eq!(
            stage.runtime.get(sorting),
            Sorting::Name,
            "a start no name has changes nothing"
        );
    }

    #[test]
    fn typing_the_start_of_a_name_highlights_its_option_while_the_list_is_open() {
        let (mut stage, select, sorting) = select();
        click(&mut stage, select);
        let overlay = overlay_over_window(&stage);

        press_characters(&mut stage, "By D");

        assert_eq!(highlighted_in(&stage, overlay), ["By date modified"]);
        assert_eq!(stage.runtime.get(sorting), Sorting::Size);
        assert_eq!(stage.overlays_of(stage.surface).len(), 1);

        assert!(press(&mut stage, Key::Enter));
        assert_eq!(stage.runtime.get(sorting), Sorting::Modified);
    }

    #[test]
    fn a_press_outside_closes_the_list_and_a_press_on_the_select_is_left_to_the_select() {
        let (mut stage, select, sorting) = select();
        let elsewhere = stage.mount(|mount| Button::new("Elsewhere").mount(mount));
        stage.frame_all();

        click(&mut stage, select);
        assert_eq!(stage.overlays_of(stage.surface).len(), 1);
        click(&mut stage, elsewhere);
        assert!(
            stage.overlays_of(stage.surface).is_empty(),
            "a press outside closes the list"
        );

        click(&mut stage, select);
        assert_eq!(stage.overlays_of(stage.surface).len(), 1);
        click(&mut stage, select);
        assert!(
            stage.overlays_of(stage.surface).is_empty(),
            "the select closes its list itself: the press on it did not, to have the click open it again"
        );
        assert_eq!(stage.runtime.get(sorting), Sorting::Size);
    }

    #[test]
    fn an_assistive_technology_opens_the_list_and_chooses_from_it() {
        let (mut stage, select, sorting) = select();

        stage.accessibility_click(select);
        stage.frame_all();

        let overlay = overlay_over_window(&stage);
        let first = rows_of(&stage, overlay)[0];
        let request = morok_core::accesskit::ActionRequest {
            action: morok_core::accesskit::Action::Click,
            target_tree: morok_core::accesskit::TreeId::ROOT,
            target_node: morok_core::accesskit::NodeId::from(first),
            data: None,
        };
        stage
            .surfaces
            .surface_mut(overlay)
            .accessibility_action(&mut stage.runtime, &request);
        stage.frame_all();

        assert_eq!(stage.runtime.get(sorting), Sorting::Name);
        assert!(stage.overlays_of(stage.surface).is_empty());
    }

    #[test]
    fn a_disabled_select_takes_no_focus_and_does_not_open() {
        let mut stage = Stage::new();
        let sorting = stage.runtime.signal(Sorting::Size);
        let select = stage.mount(move |mount| sorting_select(sorting).disabled(true).mount(mount));
        stage.frame_all();

        assert!(!stage.focus(select));
        click(&mut stage, select);
        stage.accessibility_click(select);
        stage.frame_all();

        assert!(stage.overlays_of(stage.surface).is_empty());
        assert!(stage.tree().states(select).contains(States::DISABLED));
        assert!(stage.accessible_node(select).is_disabled());
    }

    #[test]
    fn a_list_of_a_thousand_options_holds_the_rows_in_view_and_scrolls_to_the_highlight() {
        let mut stage = Stage::new();
        let chosen = stage.runtime.signal(700_usize);
        let select = stage.mount(move |mount| {
            (0..1000)
                .fold(Select::new(chosen), |select, option| {
                    select.option(option, format!("Option {option}"))
                })
                .mount(mount)
        });
        stage.frame_all();

        click(&mut stage, select);
        // The first frame of the list measures its viewport, the second scrolls it, and the
        // third binds the row slots to the rows that came into view.
        stage.frame_all();
        stage.frame_all();

        let overlay = overlay_over_window(&stage);
        let viewport = viewport_of(&stage, overlay);
        let tree = stage.tree_of(overlay);
        assert_eq!(
            (tree.border_box(viewport).height, tree.scroll_offset(viewport).y),
            (8.0 * ROW_HEIGHT, 701.0 * ROW_HEIGHT - 8.0 * ROW_HEIGHT),
            "the list shows eight rows at once, the chosen option the last of them"
        );
        let rows = rows_of(&stage, overlay);
        assert!(
            rows.len() <= 36,
            "the rows in view have elements, not the thousand options: {}",
            rows.len()
        );
        assert_eq!(highlighted_in(&stage, overlay), ["Option 700"]);
        let highlighted = stage.accessible_node_on(
            overlay,
            *rows
                .iter()
                .find(|row| tree.classes(**row).contains(&CLASSES.highlighted))
                .expect("the highlighted row has an element"),
        );
        assert_eq!(
            (highlighted.position_in_set(), highlighted.size_of_set()),
            (Some(701), Some(1000))
        );

        assert!(press(&mut stage, Key::End));
        stage.frame_all();
        stage.frame_all();

        assert_eq!(rows_of(&stage, overlay), rows, "a scroll builds no row anew");
        assert_eq!(highlighted_in(&stage, overlay), ["Option 999"]);
        assert_eq!(
            stage.tree_of(overlay).scroll_offset(viewport),
            Point {
                x: 0.0,
                y: 1000.0 * ROW_HEIGHT - 8.0 * ROW_HEIGHT,
            }
        );
        assert!(press(&mut stage, Key::Enter));
        assert_eq!(stage.runtime.get(chosen), 999);
    }
}

mod combobox {
    use super::*;

    const FOLDERS: [&str; 5] = ["Applications", "Desktop", "Documents", "Downloads", "Movies"];

    struct ComboboxStage {
        stage: Stage,
        field: ElementId,
        input: ElementId,
        chevron: ElementId,
        /// What speaks the highlighted option to an assistive technology.
        voice: ElementId,
        folder: Signal<String>,
        submitted: Signal<Vec<String>>,
    }

    fn folder_combobox(folder: Signal<String>) -> Combobox {
        FOLDERS
            .iter()
            .fold(Combobox::new(folder), |combobox, name| combobox.option(*name))
            .label("Folder")
            .placeholder("Folder")
    }

    /// A combo box of five folders that holds no text, under the root of the window.
    fn combobox() -> ComboboxStage {
        let mut stage = Stage::new();
        let folder = stage.runtime.signal(String::new());
        let submitted = stage.runtime.signal(Vec::new());
        let field = stage.mount(move |mount| {
            folder_combobox(folder)
                .on_submit(move |runtime| {
                    let text = runtime.get(folder);
                    runtime.update(submitted, |submitted| submitted.push(text));
                })
                .mount(mount)
        });
        stage.frame_all();
        let [input, chevron, voice] = stage.tree().children(field) else {
            panic!("a combo box holds its line, a chevron and the voice of its list");
        };
        let (input, chevron, voice) = (*input, *chevron, *voice);
        ComboboxStage {
            stage,
            field,
            input,
            chevron,
            voice,
            folder,
            submitted,
        }
    }

    /// The user types the text into the focused line, as the input client of a window does.
    fn type_text(stage: &mut Stage, text: &str) {
        stage
            .surfaces
            .surface_mut(stage.surface)
            .insert_text(&mut stage.runtime, text, None);
        stage.frame_all();
    }

    #[test]
    fn a_combo_box_is_the_field_of_the_kit_with_a_chevron() {
        let ComboboxStage {
            mut stage,
            field,
            input,
            chevron,
            ..
        } = combobox();
        let tree = stage.tree();

        assert_eq!(tree.classes(field), [CLASSES.field]);
        assert_eq!(tree.classes(input), [CLASSES.field_input]);
        assert_eq!(tree.role(input), morok_core::Role::TextInput);
        assert_eq!(tree.classes(chevron), [CLASSES.combobox_chevron]);
        let node = stage.accessible_node(input);
        assert_eq!(
            (node.role(), node.label(), node.is_expanded()),
            (AccessibleRole::EditableComboBox, Some("Folder"), Some(false))
        );
        assert!(stage.overlays_of(stage.surface).is_empty());
    }

    #[test]
    fn typing_in_the_field_drops_down_the_options_whose_names_hold_the_text() {
        let ComboboxStage {
            mut stage,
            field,
            input,
            folder,
            ..
        } = combobox();
        click(&mut stage, input);
        assert_eq!(stage.tree().focused(), Some(input));
        assert!(
            stage.overlays_of(stage.surface).is_empty(),
            "a click into the field opens no list"
        );

        type_text(&mut stage, "DO");

        assert_eq!(
            stage.runtime.get(folder),
            "DO",
            "the text is written as the user typed it"
        );
        let overlay = overlay_over_window(&stage);
        assert_eq!(
            names_of(&stage, overlay),
            ["Documents", "Downloads"],
            "the case of the text does not matter"
        );
        assert!(highlighted_in(&stage, overlay).is_empty(), "typing highlights no row");
        let geometry = stage
            .surfaces
            .overlay_geometry(overlay)
            .expect("the field has a box after the frame");
        let field_box = stage.tree().border_box(field);
        assert_eq!(geometry.anchor, field_box);
        assert_eq!(geometry.input, OverlayInput::Pointer);
        assert_eq!(
            (geometry.panel.width, geometry.panel.height),
            (field_box.width, 2.0 * ROW_HEIGHT + PANEL_EDGES)
        );
        let viewport = viewport_of(&stage, overlay);
        let list = stage.accessible_node_on(overlay, viewport);
        assert_eq!((list.role(), list.label()), (AccessibleRole::ListBox, Some("Folder")));
        let node = stage.accessible_node(input);
        assert_eq!((node.value(), node.is_expanded()), (Some("DO"), Some(true)));

        type_text(&mut stage, "w");
        assert_eq!(
            names_of(&stage, overlay),
            ["Downloads"],
            "the open list follows the text"
        );
        assert_eq!(
            stage
                .surfaces
                .overlay_geometry(overlay)
                .expect("the field has a box")
                .panel
                .size(),
            Size {
                width: field_box.width,
                height: ROW_HEIGHT + PANEL_EDGES,
            }
        );

        type_text(&mut stage, "x");
        assert_eq!(stage.runtime.get(folder), "DOwx");
        assert!(
            stage.overlays_of(stage.surface).is_empty(),
            "a text no name holds leaves no list to show"
        );
        assert_eq!(stage.accessible_node(input).is_expanded(), Some(false));
    }

    #[test]
    fn a_name_holds_the_text_anywhere_inside_it() {
        let ComboboxStage { mut stage, input, .. } = combobox();
        click(&mut stage, input);

        type_text(&mut stage, "o");

        let overlay = overlay_over_window(&stage);
        assert_eq!(
            names_of(&stage, overlay),
            ["Applications", "Desktop", "Documents", "Downloads", "Movies"]
        );
        type_text(&mut stage, "c");
        assert_eq!(names_of(&stage, overlay), ["Documents"]);
    }

    #[test]
    fn the_arrows_move_the_highlight_from_the_field_and_return_writes_the_name() {
        let ComboboxStage {
            mut stage,
            input,
            folder,
            submitted,
            ..
        } = combobox();
        assert!(stage.focus(input));

        assert!(
            press(&mut stage, Key::ArrowDown),
            "the arrow opens the list of an empty field"
        );
        let overlay = overlay_over_window(&stage);
        assert_eq!(names_of(&stage, overlay), FOLDERS);
        assert_eq!(highlighted_in(&stage, overlay), ["Applications"]);
        assert!(press(&mut stage, Key::ArrowDown));
        assert!(press(&mut stage, Key::ArrowDown));
        assert!(press(&mut stage, Key::ArrowDown));
        assert!(press(&mut stage, Key::ArrowUp));
        assert_eq!(highlighted_in(&stage, overlay), ["Documents"]);
        assert_eq!(stage.runtime.get(folder), "", "a highlight writes nothing to the field");
        assert_eq!(stage.tree().focused(), Some(input), "the keyboard stays in the field");

        assert!(press(&mut stage, Key::Enter));

        assert_eq!(stage.runtime.get(folder), "Documents");
        assert_eq!(stage.tree().text(input), "Documents", "the line shows the name");
        assert!(stage.overlays_of(stage.surface).is_empty());
        assert_eq!(stage.tree().focused(), Some(input));
        assert!(
            stage.runtime.get(submitted).is_empty(),
            "Return that chose a row submits nothing"
        );
    }

    #[test]
    fn the_up_arrow_opens_the_list_with_the_highlight_on_its_last_row() {
        let ComboboxStage {
            mut stage,
            input,
            folder,
            ..
        } = combobox();
        click(&mut stage, input);
        type_text(&mut stage, "d");
        let overlay = overlay_over_window(&stage);
        assert_eq!(names_of(&stage, overlay), ["Desktop", "Documents", "Downloads"]);

        assert!(press(&mut stage, Key::ArrowUp));
        assert_eq!(highlighted_in(&stage, overlay), ["Downloads"]);
        assert!(press(&mut stage, Key::Enter));

        assert_eq!(stage.runtime.get(folder), "Downloads");
    }

    #[test]
    fn a_click_on_a_row_writes_its_name_to_the_field() {
        let ComboboxStage {
            mut stage,
            input,
            folder,
            ..
        } = combobox();
        click(&mut stage, input);
        type_text(&mut stage, "d");
        let overlay = overlay_over_window(&stage);
        let rows = rows_of(&stage, overlay);

        point_at(&mut stage, overlay, rows[1]);
        assert_eq!(highlighted_in(&stage, overlay), ["Documents"]);
        click_row(&mut stage, overlay, rows[1]);

        assert_eq!(stage.runtime.get(folder), "Documents");
        assert_eq!(stage.tree().text(input), "Documents");
        assert!(stage.overlays_of(stage.surface).is_empty());
        assert_eq!(stage.tree().focused(), Some(input), "the keyboard stays in the field");
    }

    #[test]
    fn escape_and_a_press_outside_close_the_list_and_keep_the_text() {
        let ComboboxStage {
            mut stage,
            input,
            folder,
            ..
        } = combobox();
        let elsewhere = stage.mount(|mount| Button::new("Elsewhere").mount(mount));
        stage.frame_all();
        click(&mut stage, input);
        type_text(&mut stage, "do");
        assert_eq!(stage.overlays_of(stage.surface).len(), 1);

        assert!(press(&mut stage, Key::Escape));
        assert!(stage.overlays_of(stage.surface).is_empty());
        assert_eq!(stage.runtime.get(folder), "do");
        assert!(
            !press(&mut stage, Key::Escape),
            "Escape is nothing to a field whose list is closed"
        );

        type_text(&mut stage, "w");
        assert_eq!(stage.overlays_of(stage.surface).len(), 1);
        click(&mut stage, input);
        assert_eq!(
            stage.overlays_of(stage.surface).len(),
            1,
            "a press on the field is not a press outside its list"
        );
        click(&mut stage, elsewhere);
        assert!(stage.overlays_of(stage.surface).is_empty());
        assert_eq!(stage.runtime.get(folder), "dow");
    }

    #[test]
    fn return_with_no_row_highlighted_submits_the_text_and_closes_the_list() {
        let ComboboxStage {
            mut stage,
            input,
            submitted,
            ..
        } = combobox();
        click(&mut stage, input);
        type_text(&mut stage, "do");
        assert_eq!(stage.overlays_of(stage.surface).len(), 1);

        assert!(press(&mut stage, Key::Enter));

        assert_eq!(stage.runtime.get(submitted), ["do"]);
        assert!(stage.overlays_of(stage.surface).is_empty());
    }

    #[test]
    fn the_chevron_opens_the_list_with_every_option_closes_it_and_gives_the_field_the_keyboard() {
        let ComboboxStage {
            mut stage,
            input,
            chevron,
            folder,
            ..
        } = combobox();
        stage.runtime.set(folder, String::from("Movies"));
        stage.frame_all();
        assert_eq!(stage.tree().focused(), None);

        click(&mut stage, chevron);

        let overlay = overlay_over_window(&stage);
        assert_eq!(
            names_of(&stage, overlay),
            FOLDERS,
            "the chevron shows every option, whatever the text"
        );
        assert_eq!(
            highlighted_in(&stage, overlay),
            ["Movies"],
            "the option the text names is highlighted"
        );
        let row = rows_of(&stage, overlay)[4];
        assert!(stage.tree_of(overlay).states(row).contains(States::CHECKED));
        assert_eq!(
            stage.tree().focused(),
            Some(input),
            "the keys of an open list are the field's"
        );

        click(&mut stage, chevron);
        assert!(stage.overlays_of(stage.surface).is_empty());

        stage.runtime.set(folder, String::from("Pictures"));
        stage.frame_all();
        click(&mut stage, chevron);
        let overlay = overlay_over_window(&stage);
        assert_eq!(
            names_of(&stage, overlay),
            FOLDERS,
            "a text that names no option still has every option on the chevron"
        );
        assert!(highlighted_in(&stage, overlay).is_empty());
    }

    #[test]
    fn the_next_edit_after_the_chevron_filters_the_list_again() {
        let ComboboxStage {
            mut stage,
            input,
            chevron,
            folder,
            ..
        } = combobox();
        click(&mut stage, input);
        type_text(&mut stage, "d");
        click(&mut stage, chevron);
        assert!(
            stage.overlays_of(stage.surface).is_empty(),
            "the chevron closes the list on screen"
        );

        click(&mut stage, chevron);
        let overlay = overlay_over_window(&stage);
        assert_eq!(names_of(&stage, overlay), FOLDERS);

        type_text(&mut stage, "o");

        let overlay = overlay_over_window(&stage);
        assert_eq!(stage.runtime.get(folder), "do");
        assert_eq!(names_of(&stage, overlay), ["Documents", "Downloads"]);
    }

    #[test]
    fn the_field_speaks_the_highlighted_option_with_its_place_and_falls_silent_when_the_list_closes() {
        let ComboboxStage {
            mut stage,
            input,
            voice,
            ..
        } = combobox();
        let node = stage.accessible_node(voice);
        assert_eq!(
            (node.role(), node.live(), node.label()),
            (AccessibleRole::Status, Some(Live::Polite), None),
            "the voice of the list says nothing while the list is closed"
        );
        assert!(stage.focus(input));

        assert!(press(&mut stage, Key::ArrowDown));
        assert_eq!(stage.accessible_node(voice).label(), Some("Applications, 1 of 5"));
        assert!(press(&mut stage, Key::ArrowDown));
        assert!(press(&mut stage, Key::ArrowDown));
        assert_eq!(stage.accessible_node(voice).label(), Some("Documents, 3 of 5"));
        assert!(press(&mut stage, Key::ArrowUp));
        assert_eq!(stage.accessible_node(voice).label(), Some("Desktop, 2 of 5"));

        assert!(press(&mut stage, Key::Escape));

        assert_eq!(
            stage.accessible_node(voice).label(),
            None,
            "closing the list clears what the field says"
        );
    }

    #[test]
    fn a_disabled_combo_box_takes_no_focus_and_does_not_open() {
        let mut stage = Stage::new();
        let folder = stage.runtime.signal(String::new());
        let field = stage.mount(move |mount| folder_combobox(folder).disabled(true).mount(mount));
        stage.frame_all();
        let [input, chevron, _] = stage.tree().children(field) else {
            panic!("a combo box holds its line, a chevron and the voice of its list");
        };
        let (input, chevron) = (*input, *chevron);

        assert!(!stage.focus(input));
        click(&mut stage, chevron);

        assert!(stage.overlays_of(stage.surface).is_empty());
        assert!(stage.tree().states(field).contains(States::DISABLED));
    }
}
