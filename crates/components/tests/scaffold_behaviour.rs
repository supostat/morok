mod support;

use morok_components::styles::CLASSES;
use morok_components::{
    Button, IconButton, KitIcon, SearchField, Sidebar, SidebarItem, SplitAxis, SplitPane, SplitView, StatusBar, Toolbar,
};
use morok_core::accesskit::{Action, ActionData, Orientation, Role as AccessibleRole};
use morok_core::{ElementId, Key, KeyChord, Modifiers, Point, PointerButton, PointerInputKind, Role};
use morok_reactive::Signal;
use morok_style::States;
use support::Stage;

/// A key goes down and up with no modifier held, and the frame that follows runs.
fn press(stage: &mut Stage, key: Key) -> bool {
    let was_used = stage.press_key(key, Modifiers::NONE);
    stage.frame();
    was_used
}

mod split_view {
    use super::*;

    /// How far an arrow key moves a divider.
    const STEP: f32 = 16.0;

    struct SplitStage {
        stage: Stage,
        axis: SplitAxis,
        view: ElementId,
        /// The panes and the dividers between them, in their order along the view.
        parts: [ElementId; 5],
        first_length: Signal<f32>,
        first_is_collapsed: Signal<bool>,
        last_length: Signal<f32>,
    }

    /// A view along `axis` that fills the stage, 400 px wide and 300 px tall: a first pane
    /// 120 px long that folds, between 80 and 200 px; the pane that takes the rest; a last
    /// pane 100 px long, between 60 and 160 px.
    fn split_view(axis: SplitAxis) -> SplitStage {
        let mut stage = Stage::new();
        let first_length = stage.runtime.signal(120.0_f32);
        let first_is_collapsed = stage.runtime.signal(false);
        let last_length = stage.runtime.signal(100.0_f32);
        let view = stage.mount(move |mount| {
            SplitView::new()
                .axis(axis)
                .pane(
                    SplitPane::sized(first_length, 80.0..=200.0, |mount| mount.text("Sidebar").id())
                        .collapsible(first_is_collapsed),
                )
                .pane(SplitPane::flexible(|mount| mount.text("Files").id()))
                .pane(SplitPane::sized(last_length, 60.0..=160.0, |mount| {
                    mount.text("Inspector").id()
                }))
                .label("Panes")
                .mount(mount)
        });
        stage.frame();
        let parts = stage
            .children(view)
            .try_into()
            .expect("three panes lie around two dividers");
        SplitStage {
            stage,
            axis,
            view,
            parts,
            first_length,
            first_is_collapsed,
            last_length,
        }
    }

    impl SplitStage {
        /// Where each part starts along the axis and how long it is, after the last frame.
        fn spans(&self) -> [(f32, f32); 5] {
            self.parts.map(|part| {
                let part_box = self.stage.tree().border_box(part);
                match self.axis {
                    SplitAxis::Horizontal => (part_box.x, part_box.width),
                    SplitAxis::Vertical => (part_box.y, part_box.height),
                }
            })
        }

        /// The pointer at `along` on the axis, halfway across the view.
        fn pointer(&mut self, kind: PointerInputKind, along: f32) {
            let position = match self.axis {
                SplitAxis::Horizontal => Point { x: along, y: 150.0 },
                SplitAxis::Vertical => Point { x: 200.0, y: along },
            };
            self.stage.pointer_on(self.stage.surface, kind, position);
            self.stage.frame();
        }

        fn press_at(&mut self, along: f32) {
            let pressed = PointerInputKind::Pressed {
                button: PointerButton::Primary,
                click_count: 1,
            };
            self.pointer(pressed, along);
        }

        fn release_at(&mut self, along: f32) {
            let released = PointerInputKind::Released {
                button: PointerButton::Primary,
                click_count: 1,
            };
            self.pointer(released, along);
        }

        /// How long the signals of the first pane and of the last one say the two are.
        fn lengths(&mut self) -> (f32, f32) {
            (
                self.stage.runtime.get(self.first_length),
                self.stage.runtime.get(self.last_length),
            )
        }

        fn first_is_collapsed(&mut self) -> bool {
            self.stage.runtime.get(self.first_is_collapsed)
        }
    }

    #[test]
    fn the_panes_lie_side_by_side_with_a_divider_between_every_two() {
        let mut split = split_view(SplitAxis::Horizontal);
        let tree = split.stage.tree();
        let [sidebar, sidebar_divider, files, inspector_divider, inspector] = split.parts;

        assert_eq!(tree.classes(split.view), [CLASSES.split, CLASSES.horizontal]);
        assert_eq!(tree.classes(sidebar), [CLASSES.split_pane]);
        assert_eq!(tree.classes(files), [CLASSES.split_pane, CLASSES.flexible]);
        assert_eq!(tree.classes(inspector), [CLASSES.split_pane]);
        assert_eq!(
            tree.classes(sidebar_divider),
            [CLASSES.split_divider, CLASSES.horizontal]
        );
        assert_eq!(
            tree.classes(inspector_divider),
            [CLASSES.split_divider, CLASSES.horizontal]
        );
        assert_eq!(
            split.spans(),
            [(0.0, 120.0), (120.0, 1.0), (121.0, 178.0), (299.0, 1.0), (300.0, 100.0)],
            "the flexible pane takes the room the sized panes and the dividers leave"
        );
        let node = split.stage.accessible_node(sidebar_divider);
        assert_eq!((node.role(), node.label()), (AccessibleRole::Splitter, Some("Panes")));
        assert_eq!(
            node.orientation(),
            Some(Orientation::Vertical),
            "the line between panes side by side stands upright"
        );
        assert_eq!(
            (node.numeric_value(), node.min_numeric_value(), node.max_numeric_value()),
            (Some(120.0), Some(0.0), Some(200.0)),
            "the divider of a pane that folds goes down to no width"
        );
        let node = split.stage.accessible_node(inspector_divider);
        assert_eq!(
            (node.numeric_value(), node.min_numeric_value(), node.max_numeric_value()),
            (Some(100.0), Some(60.0), Some(160.0))
        );
    }

    #[test]
    fn a_dragged_divider_changes_the_width_of_its_pane_within_the_limits() {
        let mut split = split_view(SplitAxis::Horizontal);

        split.press_at(120.5);
        split.pointer(PointerInputKind::Moved, 170.5);
        assert_eq!(
            (split.lengths().0, split.spans()[0], split.spans()[2]),
            (170.0, (0.0, 170.0), (171.0, 128.0)),
            "the pane before a divider grows as the divider goes right"
        );

        split.pointer(PointerInputKind::Moved, 320.0);
        assert_eq!(
            split.lengths(),
            (200.0, 100.0),
            "the divider stops at the widest its pane goes"
        );
        split.release_at(320.0);
        split.pointer(PointerInputKind::Moved, 150.0);
        assert_eq!(
            split.lengths(),
            (200.0, 100.0),
            "a pointer that let the divider go moves it no longer"
        );

        // The inspector is 100 px wide and its divider lies at 299 px.
        split.press_at(299.5);
        split.pointer(PointerInputKind::Moved, 269.5);
        assert_eq!(
            (split.lengths().1, split.spans()[4]),
            (130.0, (270.0, 130.0)),
            "the pane after a divider grows as the divider goes left"
        );
        split.pointer(PointerInputKind::Moved, 390.0);
        assert_eq!(
            split.lengths(),
            (200.0, 60.0),
            "a pane that does not fold stops at its narrowest"
        );
        split.release_at(390.0);
    }

    #[test]
    fn a_drag_far_past_the_narrowest_width_folds_a_pane_that_can_fold() {
        let mut split = split_view(SplitAxis::Horizontal);
        let sidebar = split.parts[0];

        split.press_at(120.5);
        split.pointer(PointerInputKind::Moved, 60.5);
        assert_eq!(
            (split.lengths().0, split.first_is_collapsed()),
            (80.0, false),
            "between its narrowest width and half of it the pane keeps the narrowest"
        );

        split.pointer(PointerInputKind::Moved, 30.5);
        assert!(split.first_is_collapsed());
        assert_eq!(
            split.stage.tree().classes(sidebar),
            [CLASSES.split_pane, CLASSES.collapsed]
        );
        assert_eq!(
            (split.spans()[1], split.spans()[2]),
            ((0.0, 1.0), (1.0, 298.0)),
            "a folded pane leaves its room to the flexible one"
        );
        assert_eq!(split.stage.accessible_node(split.parts[1]).numeric_value(), Some(0.0));

        split.pointer(PointerInputKind::Moved, 100.5);
        assert_eq!(
            (split.first_is_collapsed(), split.lengths().0),
            (false, 100.0),
            "the same drag unfolds it on the way back"
        );
        split.release_at(100.5);
        assert_eq!(split.spans()[0], (0.0, 100.0));
    }

    #[test]
    fn the_arrow_keys_move_a_focused_divider_by_steps_and_return_folds_its_pane() {
        let mut split = split_view(SplitAxis::Horizontal);
        let [_, sidebar_divider, _, inspector_divider, _] = split.parts;
        assert!(split.stage.focus(sidebar_divider));

        assert!(press(&mut split.stage, Key::ArrowRight));
        assert_eq!(split.lengths(), (120.0 + STEP, 100.0));
        for _ in 0..4 {
            assert!(press(&mut split.stage, Key::ArrowLeft));
        }
        assert_eq!(
            (split.lengths().0, split.first_is_collapsed()),
            (80.0, false),
            "the arrows stop at the narrowest width first"
        );
        assert!(press(&mut split.stage, Key::ArrowLeft));
        assert!(
            split.first_is_collapsed(),
            "and narrow a pane that folds away from there"
        );
        assert!(press(&mut split.stage, Key::ArrowRight));
        assert_eq!(
            (split.first_is_collapsed(), split.lengths().0),
            (false, 80.0),
            "the arrow back unfolds it at its narrowest"
        );

        assert!(press(&mut split.stage, Key::Enter));
        assert!(split.first_is_collapsed(), "Return folds the pane");
        assert!(press(&mut split.stage, Key::Space));
        assert!(!split.first_is_collapsed(), "and Space unfolds it");

        assert!(split.stage.focus(inspector_divider));
        assert!(press(&mut split.stage, Key::ArrowLeft));
        assert_eq!(
            split.lengths(),
            (80.0, 100.0 + STEP),
            "the arrow that moves the divider away from a pane after it widens that pane"
        );
        for _ in 0..5 {
            assert!(press(&mut split.stage, Key::ArrowRight));
        }
        assert_eq!(split.lengths(), (80.0, 60.0));
        assert!(
            !press(&mut split.stage, Key::Enter),
            "Return is nothing to the divider of a pane that does not fold"
        );
        assert!(
            !press(&mut split.stage, Key::ArrowDown),
            "an arrow across the axis is nothing to a divider"
        );
    }

    #[test]
    fn an_assistive_technology_moves_a_divider_and_sets_the_width_of_its_pane() {
        let mut split = split_view(SplitAxis::Horizontal);
        let sidebar_divider = split.parts[1];

        split
            .stage
            .accessibility_action(sidebar_divider, Action::Increment, None);
        assert_eq!(split.lengths(), (120.0 + STEP, 100.0));
        split
            .stage
            .accessibility_action(sidebar_divider, Action::Decrement, None);
        assert_eq!(split.lengths(), (120.0, 100.0));

        let asked = Some(ActionData::NumericValue(150.0));
        split
            .stage
            .accessibility_action(sidebar_divider, Action::SetValue, asked);
        split.stage.frame();
        assert_eq!(split.lengths(), (150.0, 100.0));
        assert_eq!(
            split.stage.accessible_node(sidebar_divider).numeric_value(),
            Some(150.0)
        );

        let too_wide = Some(ActionData::NumericValue(900.0));
        split
            .stage
            .accessibility_action(sidebar_divider, Action::SetValue, too_wide);
        assert_eq!(split.lengths(), (200.0, 100.0));
    }

    #[test]
    fn the_application_folds_a_pane_by_its_signal() {
        let mut split = split_view(SplitAxis::Horizontal);
        let sidebar = split.parts[0];

        split.stage.runtime.set(split.first_is_collapsed, true);
        split.stage.frame();

        assert_eq!(
            split.stage.tree().classes(sidebar),
            [CLASSES.split_pane, CLASSES.collapsed]
        );
        assert_eq!(split.spans()[2], (1.0, 298.0));
        assert_eq!(split.lengths(), (120.0, 100.0), "a folded pane remembers its width");
    }

    #[test]
    fn a_vertical_view_stacks_its_panes_with_a_level_divider_between_every_two() {
        let mut split = split_view(SplitAxis::Vertical);
        let tree = split.stage.tree();
        let [_, top_divider, _, bottom_divider, _] = split.parts;

        assert_eq!(tree.classes(split.view), [CLASSES.split, CLASSES.vertical]);
        assert_eq!(tree.classes(top_divider), [CLASSES.split_divider, CLASSES.vertical]);
        let grip = tree.children(top_divider)[0];
        assert_eq!(tree.classes(grip), [CLASSES.split_grip, CLASSES.vertical]);
        assert_eq!(
            split.spans(),
            [(0.0, 120.0), (120.0, 1.0), (121.0, 78.0), (199.0, 1.0), (200.0, 100.0)],
            "the panes stand one above another and the flexible one takes the height left"
        );
        for part in split.parts {
            let part_box = tree.border_box(part);
            assert_eq!(
                (part_box.x, part_box.width),
                (0.0, 400.0),
                "every part spans the view from side to side"
            );
        }
        assert_eq!(
            (tree.border_box(grip).y, tree.border_box(grip).height),
            (117.0, 7.0),
            "the pointer takes a level line by the room above and below it"
        );
        let node = split.stage.accessible_node(top_divider);
        assert_eq!((node.role(), node.label()), (AccessibleRole::Splitter, Some("Panes")));
        assert_eq!(
            node.orientation(),
            Some(Orientation::Horizontal),
            "the line between panes one above another lies level"
        );
        assert_eq!(
            (node.numeric_value(), node.min_numeric_value(), node.max_numeric_value()),
            (Some(120.0), Some(0.0), Some(200.0))
        );
        let node = split.stage.accessible_node(bottom_divider);
        assert_eq!(
            (node.numeric_value(), node.orientation()),
            (Some(100.0), Some(Orientation::Horizontal))
        );
    }

    #[test]
    fn a_dragged_divider_changes_the_height_of_its_pane_within_the_limits() {
        let mut split = split_view(SplitAxis::Vertical);

        split.press_at(120.5);
        split.pointer(PointerInputKind::Moved, 170.5);
        assert_eq!(
            (split.lengths().0, split.spans()[0], split.spans()[2]),
            (170.0, (0.0, 170.0), (171.0, 28.0)),
            "the pane above a divider grows as the divider goes down"
        );
        split.pointer(PointerInputKind::Moved, 290.0);
        assert_eq!(
            split.lengths(),
            (200.0, 100.0),
            "the divider stops at the tallest its pane goes"
        );
        split.pointer(PointerInputKind::Moved, 150.5);
        split.release_at(150.5);
        assert_eq!(split.lengths(), (150.0, 100.0));

        // The last pane is 100 px tall and its divider lies at 199 px.
        split.press_at(199.5);
        split.pointer(PointerInputKind::Moved, 169.5);
        assert_eq!(
            (split.lengths().1, split.spans()[4]),
            (130.0, (170.0, 130.0)),
            "the pane below a divider grows as the divider goes up"
        );
        split.pointer(PointerInputKind::Moved, 295.0);
        assert_eq!(
            split.lengths(),
            (150.0, 60.0),
            "a pane that does not fold stops at its shortest"
        );
        split.release_at(295.0);
    }

    #[test]
    fn a_drag_far_past_the_shortest_height_folds_a_pane_that_can_fold() {
        let mut split = split_view(SplitAxis::Vertical);
        let top = split.parts[0];

        split.press_at(120.5);
        split.pointer(PointerInputKind::Moved, 60.5);
        assert_eq!(
            (split.lengths().0, split.first_is_collapsed()),
            (80.0, false),
            "between its shortest height and half of it the pane keeps the shortest"
        );

        split.pointer(PointerInputKind::Moved, 30.5);
        assert!(split.first_is_collapsed());
        assert_eq!(split.stage.tree().classes(top), [CLASSES.split_pane, CLASSES.collapsed]);
        assert_eq!(
            (split.spans()[1], split.spans()[2]),
            ((0.0, 1.0), (1.0, 198.0)),
            "a folded pane leaves its height to the flexible one"
        );

        split.pointer(PointerInputKind::Moved, 100.5);
        assert_eq!(
            (split.first_is_collapsed(), split.lengths().0),
            (false, 100.0),
            "the same drag unfolds it on the way back"
        );
        split.release_at(100.5);
        assert_eq!(split.spans()[0], (0.0, 100.0));
    }

    #[test]
    fn arrow_down_and_arrow_up_move_a_focused_divider_of_a_vertical_view() {
        let mut split = split_view(SplitAxis::Vertical);
        let [_, top_divider, _, bottom_divider, _] = split.parts;
        assert!(split.stage.focus(top_divider));

        assert!(press(&mut split.stage, Key::ArrowDown));
        assert_eq!(split.lengths(), (120.0 + STEP, 100.0));
        for _ in 0..4 {
            assert!(press(&mut split.stage, Key::ArrowUp));
        }
        assert_eq!(
            (split.lengths().0, split.first_is_collapsed()),
            (80.0, false),
            "the arrows stop at the shortest height first"
        );
        assert!(press(&mut split.stage, Key::ArrowUp));
        assert!(split.first_is_collapsed(), "and fold a pane that folds from there");
        assert!(press(&mut split.stage, Key::ArrowDown));
        assert_eq!((split.first_is_collapsed(), split.lengths().0), (false, 80.0));
        assert!(
            !press(&mut split.stage, Key::ArrowRight) && !press(&mut split.stage, Key::ArrowLeft),
            "the arrows across the axis are nothing to a level divider"
        );

        assert!(split.stage.focus(bottom_divider));
        assert!(press(&mut split.stage, Key::ArrowUp));
        assert_eq!(
            split.lengths(),
            (80.0, 100.0 + STEP),
            "the arrow that moves the divider up lengthens the pane below it"
        );
        for _ in 0..5 {
            assert!(press(&mut split.stage, Key::ArrowDown));
        }
        assert_eq!(split.lengths(), (80.0, 60.0));
    }
}

mod sidebar {
    use super::*;

    /// A sidebar of two groups of two places each, with the first place selected; returns
    /// the sidebar and its four items.
    fn sidebar() -> (Stage, ElementId, [ElementId; 4], Signal<&'static str>) {
        let mut stage = Stage::new();
        let place = stage.runtime.signal("macintosh");
        let sidebar = stage.mount(move |mount| {
            Sidebar::new(place)
                .heading("VOLUMES")
                .item(
                    SidebarItem::new("macintosh", "Macintosh HD")
                        .icon(KitIcon::Disk)
                        .detail("1 TB"),
                )
                .item(
                    SidebarItem::new("archive", "Archive")
                        .icon(KitIcon::Disk)
                        .detail("4 TB"),
                )
                .heading("RECOMMENDATIONS")
                .item(SidebarItem::new("caches", "Build caches").icon(KitIcon::Bolt))
                .item(SidebarItem::new("duplicates", "Duplicates"))
                .label("Places")
                .mount(mount)
        });
        stage.frame();
        let rows = stage.children(sidebar);
        (stage, sidebar, [rows[1], rows[2], rows[4], rows[5]], place)
    }

    #[test]
    fn a_sidebar_is_a_column_of_headings_and_items_with_the_selected_item_marked() {
        let (mut stage, sidebar, items, _) = sidebar();
        let tree = stage.tree();
        let rows = tree.children(sidebar);

        assert_eq!(tree.classes(sidebar), [CLASSES.sidebar]);
        assert_eq!(rows.len(), 6);
        assert_eq!(
            (tree.text(rows[0]), tree.classes(rows[0])),
            ("VOLUMES", [CLASSES.sidebar_heading].as_slice())
        );
        assert_eq!(tree.text(rows[3]), "RECOMMENDATIONS");
        let [icon, name, detail] = tree.children(items[0]) else {
            panic!("an item with an icon and a detail holds the three");
        };
        assert_eq!(
            (tree.role(*icon), tree.classes(*icon)),
            (Role::Icon, [CLASSES.sidebar_item_icon].as_slice())
        );
        assert_eq!(tree.text(*name), "Macintosh HD");
        assert_eq!(
            (tree.text(*detail), tree.classes(*detail)),
            ("1 TB", [CLASSES.sidebar_item_detail].as_slice())
        );
        assert_eq!(
            tree.children(items[3]).len(),
            1,
            "an item with a name alone holds the name"
        );
        assert!(tree.states(items[0]).contains(States::CHECKED));
        assert!(!tree.states(items[1]).contains(States::CHECKED));

        let node = stage.accessible_node(sidebar);
        assert_eq!((node.role(), node.label()), (AccessibleRole::ListBox, Some("Places")));
        let selected = stage.accessible_node(items[0]);
        assert_eq!(
            (selected.role(), selected.label(), selected.is_selected()),
            (AccessibleRole::ListBoxOption, Some("Macintosh HD"), Some(true))
        );
        assert_eq!(stage.accessible_node(items[1]).is_selected(), Some(false));
    }

    #[test]
    fn a_click_the_arrow_keys_and_an_assistive_technology_select_an_item() {
        let (mut stage, _, items, place) = sidebar();

        stage.click(items[2]);
        stage.frame();
        assert_eq!(stage.runtime.get(place), "caches");
        assert!(stage.tree().states(items[2]).contains(States::CHECKED));
        assert_eq!(stage.tree().focused(), Some(items[2]));

        assert!(press(&mut stage, Key::ArrowDown));
        assert_eq!(stage.runtime.get(place), "duplicates");
        assert_eq!(stage.tree().focused(), Some(items[3]), "focus follows the selection");
        assert!(press(&mut stage, Key::ArrowUp));
        assert!(press(&mut stage, Key::ArrowUp));
        assert_eq!(
            stage.runtime.get(place),
            "archive",
            "the arrows go over a heading to the item before it"
        );

        stage.accessibility_click(items[0]);
        stage.frame();
        assert_eq!(stage.runtime.get(place), "macintosh");
    }

    #[test]
    fn the_tab_key_stops_on_the_selected_item_alone() {
        let (mut stage, _, items, place) = sidebar();
        let after = stage.mount(|mount| Button::new("After").mount(mount));
        stage.runtime.set(place, "archive");
        stage.frame();

        assert!(press(&mut stage, Key::Tab));
        assert_eq!(stage.tree().focused(), Some(items[1]));
        assert!(press(&mut stage, Key::Tab));
        assert_eq!(
            stage.tree().focused(),
            Some(after),
            "the other items are no stops of the Tab key"
        );
    }
}

mod toolbar {
    use super::*;

    struct ToolbarStage {
        stage: Stage,
        toolbar: ElementId,
        sidebar_button: ElementId,
        /// The line of the search field.
        search_line: ElementId,
        go_button: ElementId,
        presses: Signal<u32>,
    }

    /// A toolbar of a button with an icon, the title of the window and, at its far end, a
    /// search field and a button: together they leave the spacer a few of the 400 px.
    fn toolbar() -> ToolbarStage {
        let mut stage = Stage::new();
        let presses = stage.runtime.signal(0_u32);
        let query = stage.runtime.signal(String::new());
        let toolbar = stage.mount(move |mount| {
            Toolbar::new()
                .item(move |mount| {
                    IconButton::new(KitIcon::Sidebar, "Sidebar")
                        .on_press(move |runtime| runtime.update(presses, |presses| *presses += 1))
                        .mount(mount)
                })
                .title("Files")
                .spacer()
                .item(move |mount| SearchField::new(query).placeholder("Filter").mount(mount))
                .item(|mount| Button::new("Go").mount(mount))
                .label("Window")
                .mount(mount)
        });
        stage.frame();
        let [sidebar_button, _, _, search_field, go_button] = stage.children(toolbar)[..] else {
            panic!("the toolbar holds its five parts");
        };
        let search_line = *stage
            .tree()
            .children(search_field)
            .last()
            .expect("a search field holds its line last");
        ToolbarStage {
            stage,
            toolbar,
            sidebar_button,
            search_line,
            go_button,
            presses,
        }
    }

    #[test]
    fn a_toolbar_is_a_row_of_its_parts_with_the_far_end_past_the_spacer() {
        let mut bar = toolbar();
        let tree = bar.stage.tree();
        let parts = tree.children(bar.toolbar);

        assert_eq!(tree.classes(bar.toolbar), [CLASSES.toolbar]);
        assert_eq!(
            (tree.text(parts[1]), tree.classes(parts[1])),
            ("Files", [CLASSES.toolbar_title].as_slice())
        );
        assert_eq!(tree.classes(parts[2]), [CLASSES.toolbar_spacer]);
        let toolbar_box = tree.border_box(bar.toolbar);
        let last_box = tree.border_box(bar.go_button);
        assert_eq!(
            (toolbar_box.width, toolbar_box.height),
            (400.0, 44.0),
            "the bar spans the window"
        );
        assert_eq!(
            (last_box.right(), last_box.height),
            (toolbar_box.right() - 12.0, 28.0),
            "what follows the spacer stands at the far end, inside the padding"
        );
        let node = bar.stage.accessible_node(bar.toolbar);
        assert_eq!((node.role(), node.label()), (AccessibleRole::Toolbar, Some("Window")));
    }

    #[test]
    fn the_arrow_keys_move_focus_from_control_to_control() {
        let mut bar = toolbar();
        assert!(bar.stage.focus(bar.sidebar_button));

        assert!(press(&mut bar.stage, Key::ArrowRight));
        assert_eq!(
            bar.stage.tree().focused(),
            Some(bar.search_line),
            "focus goes to what takes it inside the next control, past the title and the spacer"
        );
        assert!(press(&mut bar.stage, Key::ArrowRight));
        assert_eq!(
            bar.stage.tree().focused(),
            Some(bar.search_line),
            "a text field uses the arrows itself"
        );

        assert!(bar.stage.focus(bar.go_button));
        assert!(press(&mut bar.stage, Key::ArrowLeft));
        assert_eq!(bar.stage.tree().focused(), Some(bar.search_line));
        assert!(bar.stage.focus(bar.go_button));
        assert!(press(&mut bar.stage, Key::Home));
        assert_eq!(bar.stage.tree().focused(), Some(bar.sidebar_button));
        assert!(press(&mut bar.stage, Key::ArrowLeft));
        assert_eq!(
            bar.stage.tree().focused(),
            Some(bar.sidebar_button),
            "focus stays on the first control at the start of the bar"
        );
        assert!(press(&mut bar.stage, Key::End));
        assert_eq!(bar.stage.tree().focused(), Some(bar.go_button));
    }

    #[test]
    fn the_controls_of_a_toolbar_answer_the_keyboard_and_an_assistive_technology() {
        let mut bar = toolbar();
        assert!(bar.stage.focus(bar.sidebar_button));

        assert!(press(&mut bar.stage, Key::Space));
        bar.stage.accessibility_click(bar.sidebar_button);

        assert_eq!(bar.stage.runtime.get(bar.presses), 2);
    }
}

mod status_bar {
    use super::*;

    const F5: KeyChord = KeyChord {
        key: Key::Function(5),
        modifiers: Modifiers::NONE,
    };
    const F8: KeyChord = KeyChord {
        key: Key::Function(8),
        modifiers: Modifiers::NONE,
    };

    struct StatusStage {
        stage: Stage,
        bar: ElementId,
        status: Signal<String>,
        given: Signal<Vec<&'static str>>,
    }

    /// A status bar of a line of text, two commands and, at its far end, a hint.
    fn status_bar() -> StatusStage {
        let mut stage = Stage::new();
        let status = stage.runtime.signal(String::from("Selected: 1 folder"));
        let given = stage.runtime.signal(Vec::new());
        let bar = stage.mount(move |mount| {
            StatusBar::new()
                .text(status)
                .command(F5, "Copy", move |runtime| {
                    runtime.update(given, |given| given.push("copy"));
                })
                .command(F8, "Delete", move |runtime| {
                    runtime.update(given, |given| given.push("delete"));
                })
                .spacer()
                .text("Tab — other pane")
                .label("Status")
                .mount(mount)
        });
        stage.frame();
        StatusStage {
            stage,
            bar,
            status,
            given,
        }
    }

    #[test]
    fn a_status_bar_shows_its_lines_and_its_commands_with_their_key_caps() {
        let mut status = status_bar();
        let tree = status.stage.tree();
        let [line, copy, delete, spacer, hint] = tree.children(status.bar) else {
            panic!("the status bar holds its five parts");
        };
        let (copy, delete) = (*copy, *delete);

        assert_eq!(tree.classes(status.bar), [CLASSES.statusbar]);
        assert_eq!(tree.text(*line), "Selected: 1 folder");
        assert_eq!(tree.classes(copy), [CLASSES.status_command]);
        let [key_cap, name] = tree.children(copy) else {
            panic!("a command holds the key cap of its chord and its name");
        };
        assert_eq!(tree.classes(*key_cap), [CLASSES.status_key]);
        assert_eq!(tree.text(tree.children(*key_cap)[0]), "F5");
        assert_eq!(tree.text(*name), "Copy");
        assert_eq!(tree.classes(*spacer), [CLASSES.status_spacer]);
        let bar_box = tree.border_box(status.bar);
        assert_eq!(
            (bar_box.height, tree.border_box(*hint).right()),
            (24.0, bar_box.right() - 12.0),
            "what follows the spacer stands at the far end of the bar"
        );

        let node = status.stage.accessible_node(status.bar);
        assert_eq!((node.role(), node.label()), (AccessibleRole::Status, Some("Status")));
        let command = status.stage.accessible_node(delete);
        assert_eq!(
            (command.role(), command.label()),
            (AccessibleRole::Button, Some("Delete"))
        );
    }

    #[test]
    fn a_line_of_a_status_bar_follows_its_signal() {
        let mut status = status_bar();
        let line = status.stage.children(status.bar)[0];

        status
            .stage
            .runtime
            .set(status.status, String::from("Selected: 3 files"));
        status.stage.frame();

        assert_eq!(status.stage.tree().text(line), "Selected: 3 files");
    }

    #[test]
    fn a_click_the_keyboard_and_an_assistive_technology_give_a_command() {
        let mut status = status_bar();
        let parts = status.stage.children(status.bar);
        let (copy, delete) = (parts[1], parts[2]);

        status.stage.click(copy);
        assert_eq!(status.stage.tree().focused(), Some(copy));
        assert!(press(&mut status.stage, Key::Tab));
        assert_eq!(
            status.stage.tree().focused(),
            Some(delete),
            "the commands are stops of the Tab key"
        );
        assert!(press(&mut status.stage, Key::Space));
        assert!(press(&mut status.stage, Key::Enter));
        status.stage.accessibility_click(copy);

        assert_eq!(
            status.stage.runtime.get(status.given),
            ["copy", "delete", "delete", "copy"]
        );
        assert!(
            !press(&mut status.stage, Key::Function(5)),
            "the bar answers no chord itself: the application registers the shortcut"
        );
    }
}
