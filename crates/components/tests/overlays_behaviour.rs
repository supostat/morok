mod support;

use std::time::Duration;

use morok_components::styles::CLASSES;
use morok_components::view::{Mount, SurfaceId};
use morok_components::{
    Button, ContextMenu, Dialog, KitIcon, Menu, MenuItem, Popover, Sheet, Toast, ToastStack, ToastTone, Tooltip,
};
use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{
    ElementId, Key, KeyChord, Modifiers, NativeMenuItem, OverlayAlignment, OverlayInput, OverlaySide, Point,
    PointerButton, PointerInputKind, Rect, Role, Size,
};
use morok_reactive::Signal;
use morok_style::States;
use support::Stage;

/// The room an overlay surface leaves around its panel for the shadow of the panel.
const SHADOW_ROOM: f32 = 32.0;

/// A stage with a button that opens and closes what `attach` ties to it.
fn stage_with_trigger(
    attach: impl FnOnce(&mut Mount<'_>, ElementId, Signal<bool>),
) -> (Stage, ElementId, Signal<bool>) {
    let mut stage = Stage::new();
    let is_open = stage.runtime.signal(false);
    let trigger = stage.mount(move |mount| {
        let trigger = Button::new("Open")
            .on_press(move |runtime| runtime.update(is_open, |is_open| *is_open = !*is_open))
            .mount(mount);
        attach(mount, trigger, is_open);
        trigger
    });
    stage.frame_all();
    (stage, trigger, is_open)
}

/// Clicks the element of the surface of the window and runs the frames that follow.
fn click(stage: &mut Stage, element: ElementId) {
    stage.click(element);
    stage.frame_all();
}

/// A key goes down and up on a surface, and the frames that follow run.
fn press(stage: &mut Stage, surface: SurfaceId, key: Key) -> bool {
    let was_used = stage.press_key_on(surface, key);
    stage.frame_all();
    was_used
}

/// The pointer comes to rest on an element of a surface, and the frames that follow run.
fn hover(stage: &mut Stage, surface: SurfaceId, element: ElementId) {
    let position = stage.middle_on(surface, element);
    stage.pointer_on(surface, PointerInputKind::Moved, position);
    stage.frame_all();
}

/// The only overlay opened over the surface.
fn overlay_over(stage: &Stage, surface: SurfaceId) -> SurfaceId {
    let overlays = stage.overlays_of(surface);
    assert_eq!(overlays.len(), 1, "one overlay is open over the surface");
    overlays[0]
}

/// What an overlay shows: the panel, inside the box that leaves room for its shadow.
fn panel_of(stage: &Stage, overlay: SurfaceId) -> ElementId {
    let tree = stage.tree_of(overlay);
    let [frame] = tree.children(tree.root()) else {
        panic!("an overlay surface holds one box");
    };
    assert_eq!(tree.classes(*frame), [CLASSES.overlay_surface]);
    let [panel] = tree.children(*frame) else {
        panic!("the box of an overlay surface holds the panel");
    };
    *panel
}

mod popover {
    use super::*;

    /// A popover with two buttons, tied to a button that opens and closes it.
    fn popover() -> (Stage, ElementId, Signal<bool>) {
        stage_with_trigger(|mount, trigger, is_open| {
            Popover::new(is_open)
                .title("Xcode cache")
                .message("38.4 GB in DerivedData.")
                .content(|mount| {
                    let clean_up = Button::new("Clean up").primary().mount(mount);
                    let later = Button::new("Later").mount(mount);
                    mount.element().child(clean_up).child(later).id()
                })
                .attach(mount, trigger);
        })
    }

    #[test]
    fn a_popover_opens_on_a_surface_of_its_own_placed_under_its_anchor() {
        let (mut stage, trigger, _) = popover();
        assert!(
            stage.overlays_of(stage.surface).is_empty(),
            "a closed popover has no surface"
        );

        click(&mut stage, trigger);

        let overlay = overlay_over(&stage, stage.surface);
        let geometry = stage
            .surfaces
            .overlay_geometry(overlay)
            .expect("the anchor has a box after the frame");
        assert_eq!(geometry.anchor, stage.tree().border_box(trigger));
        assert_eq!(
            (geometry.placement.side, geometry.placement.alignment),
            (OverlaySide::Below, OverlayAlignment::Start)
        );
        assert_eq!(
            geometry.input,
            OverlayInput::PointerAndKeyboard,
            "a popover takes the pointer and the keyboard"
        );
        assert_eq!(
            (geometry.panel.x, geometry.panel.y, geometry.panel.width),
            (SHADOW_ROOM, SHADOW_ROOM, 260.0)
        );
        assert_eq!(
            geometry.size,
            Size {
                width: 260.0 + 2.0 * SHADOW_ROOM,
                height: geometry.panel.height + 2.0 * SHADOW_ROOM,
            },
            "the surface of an overlay is its panel with room for the shadow on every side"
        );

        let panel = panel_of(&stage, overlay);
        let tree = stage.tree_of(overlay);
        assert_eq!(tree.classes(panel), [CLASSES.popover]);
        let [arrow, title, message, _content] = tree.children(panel) else {
            panic!("a popover holds its arrow, its title, its message and its content");
        };
        assert_eq!(tree.classes(*arrow), [CLASSES.popover_arrow]);
        assert_eq!(tree.text(*title), "Xcode cache");
        assert_eq!(tree.text(*message), "38.4 GB in DerivedData.");
        let node = stage.accessible_node_on(overlay, panel);
        assert_eq!(node.role(), AccessibleRole::Dialog);
    }

    #[test]
    fn the_keyboard_of_a_popover_stays_inside_it_and_escape_gives_it_back() {
        let (mut stage, trigger, is_open) = popover();
        click(&mut stage, trigger);
        let overlay = overlay_over(&stage, stage.surface);
        let panel = panel_of(&stage, overlay);
        let content = stage.tree_of(overlay).children(panel)[3];
        let [clean_up, later] = stage.tree_of(overlay).children(content) else {
            panic!("the content holds the two buttons");
        };
        let (clean_up, later) = (*clean_up, *later);

        assert!(press(&mut stage, overlay, Key::Tab));
        assert_eq!(stage.tree_of(overlay).focused(), Some(clean_up));
        assert!(press(&mut stage, overlay, Key::Tab));
        assert_eq!(stage.tree_of(overlay).focused(), Some(later));
        assert!(press(&mut stage, overlay, Key::Tab));
        assert_eq!(
            stage.tree_of(overlay).focused(),
            Some(clean_up),
            "after the last stop of the popover comes its first"
        );
        assert_eq!(
            stage.tree().focused(),
            Some(trigger),
            "focus under the popover stays put"
        );

        assert!(
            press(&mut stage, overlay, Key::Escape),
            "Escape that nothing used closes the popover"
        );

        assert!(!stage.runtime.get(is_open));
        assert!(stage.overlays_of(stage.surface).is_empty());
        assert!(
            !stage.surfaces.contains(overlay),
            "the surface of the popover is closed"
        );
        assert_eq!(
            stage.tree().focused(),
            Some(trigger),
            "and the keyboard is back where it was"
        );
    }

    #[test]
    fn a_press_outside_a_popover_closes_it_and_a_press_on_its_anchor_is_left_to_the_anchor() {
        let (mut stage, trigger, is_open) = popover();
        let elsewhere = stage.mount(|mount| Button::new("Elsewhere").mount(mount));
        stage.frame_all();

        click(&mut stage, trigger);
        click(&mut stage, elsewhere);
        assert!(!stage.runtime.get(is_open), "a press outside closes the popover");
        assert!(stage.overlays_of(stage.surface).is_empty());

        click(&mut stage, trigger);
        assert!(stage.runtime.get(is_open));
        click(&mut stage, trigger);
        assert!(
            !stage.runtime.get(is_open),
            "the anchor closes the popover itself: the press on it did not, to have the click open it again"
        );
        assert!(stage.overlays_of(stage.surface).is_empty());
    }

    #[test]
    fn a_press_on_the_room_around_the_panel_goes_to_the_surface_under_the_popover() {
        let (mut stage, trigger, is_open) = popover();
        click(&mut stage, trigger);
        let overlay = overlay_over(&stage, stage.surface);
        let inside_panel = stage.middle_on(overlay, panel_of(&stage, overlay));

        stage.click_on(overlay, PointerButton::Primary, inside_panel);
        stage.frame_all();
        assert!(stage.runtime.get(is_open), "a press on the panel is the popover's own");

        // The top of the overlay surface lies over the anchor: the panel is 10 px under it
        // and has 32 px of room above.
        let over_the_anchor = Point { x: 40.0, y: 20.0 };
        stage.click_on(overlay, PointerButton::Primary, over_the_anchor);
        stage.frame_all();

        assert!(
            !stage.runtime.get(is_open),
            "the click went through the room around the panel to the anchor, which closed the popover"
        );
        assert!(stage.tree().states(trigger).contains(States::HOVER));
    }

    #[test]
    fn a_popover_with_no_room_under_its_anchor_lies_over_it_with_its_arrow_below() {
        let mut stage = Stage::new();
        // Twenty buttons of 28 px put the anchor at the bottom of the 600 px of the screen.
        for _ in 0..20 {
            stage.mount(|mount| Button::new("Above").mount(mount));
        }
        let is_open = stage.runtime.signal(true);
        let trigger = stage.mount(move |mount| {
            let trigger = Button::new("Open").mount(mount);
            Popover::new(is_open).title("Xcode cache").attach(mount, trigger);
            trigger
        });

        stage.frame_all();
        stage.frame_all();

        assert_eq!(stage.tree().border_box(trigger).origin(), Point { x: 0.0, y: 560.0 });
        let overlay = overlay_over(&stage, stage.surface);
        let arrow = stage.tree_of(overlay).children(panel_of(&stage, overlay))[0];
        assert_eq!(
            stage.tree_of(overlay).classes(arrow),
            [CLASSES.popover_arrow, CLASSES.above],
            "the arrow points down at the anchor from a popover that flipped over it"
        );
    }

    #[test]
    fn a_popover_goes_with_the_element_it_is_tied_to() {
        let mut stage = Stage::new();
        let is_present = stage.runtime.signal(true);
        let is_open = stage.runtime.signal(true);
        stage.mount(move |mount| {
            mount
                .element()
                .show(is_present, move |mount| {
                    let trigger = Button::new("Open").mount(mount);
                    Popover::new(is_open).title("Xcode cache").attach(mount, trigger);
                    trigger
                })
                .id()
        });
        stage.frame_all();
        let overlay = overlay_over(&stage, stage.surface);

        stage.runtime.set(is_present, false);
        stage.frame_all();

        assert!(stage.overlays_of(stage.surface).is_empty());
        assert!(!stage.surfaces.contains(overlay));
    }
}

/// The items of the menus of the tests; a command adds its name to `chosen` when it is chosen.
fn menu_items(chosen: Signal<Vec<&'static str>>) -> Vec<MenuItem> {
    let command = |label: &'static str, name: &'static str| {
        MenuItem::new(label).on_choose(move |runtime| runtime.update(chosen, |chosen| chosen.push(name)))
    };
    vec![
        command("Quick Look", "quick look")
            .icon(KitIcon::Eye)
            .shortcut(KeyChord {
                key: Key::Space,
                modifiers: Modifiers::NONE,
            }),
        command("Copy path", "copy path")
            .icon(KitIcon::Copy)
            .shortcut(KeyChord {
                key: Key::Character('c'),
                modifiers: Modifiers::COMMAND.union(Modifiers::OPTION),
            }),
        MenuItem::submenu(
            "Open with",
            vec![command("Preview", "preview"), command("TextEdit", "textedit")],
        ),
        command("Remove download", "remove download")
            .icon(KitIcon::Cloud)
            .disabled(true),
        MenuItem::separator(),
        command("Move to Trash", "move to trash")
            .icon(KitIcon::Trash)
            .destructive(),
    ]
}

mod menu {
    use super::*;

    struct OpenMenu {
        stage: Stage,
        trigger: ElementId,
        is_open: Signal<bool>,
        chosen: Signal<Vec<&'static str>>,
        overlay: SurfaceId,
        lines: Vec<ElementId>,
    }

    /// A menu opened from its button.
    fn open_menu() -> OpenMenu {
        let mut chosen = None;
        let (mut stage, trigger, is_open) = stage_with_trigger(|mount, trigger, is_open| {
            let chosen_names = mount.runtime().signal(Vec::new());
            chosen = Some(chosen_names);
            Menu::new(is_open, menu_items(chosen_names))
                .label("File actions")
                .attach(mount, trigger);
        });
        click(&mut stage, trigger);
        let overlay = overlay_over(&stage, stage.surface);
        let lines = stage.tree_of(overlay).children(panel_of(&stage, overlay)).to_vec();
        OpenMenu {
            stage,
            trigger,
            is_open,
            chosen: chosen.expect("the menu was attached"),
            overlay,
            lines,
        }
    }

    /// The submenu opened over the menu, with its lines.
    fn submenu_of(stage: &Stage, menu: SurfaceId) -> (SurfaceId, Vec<ElementId>) {
        let submenu = overlay_over(stage, menu);
        let lines = stage.tree_of(submenu).children(panel_of(stage, submenu)).to_vec();
        (submenu, lines)
    }

    #[test]
    fn a_menu_is_a_column_of_its_lines_on_a_surface_under_its_anchor() {
        let mut menu = open_menu();
        let panel = panel_of(&menu.stage, menu.overlay);
        let tree = menu.stage.tree_of(menu.overlay);

        assert_eq!(tree.classes(panel), [CLASSES.menu]);
        assert_eq!(menu.lines.len(), 6);
        let [icon, label, shortcut] = tree.children(menu.lines[1]) else {
            panic!("a command with an icon and a shortcut holds the three");
        };
        assert_eq!(
            (tree.role(*icon), tree.classes(*icon)),
            (Role::Icon, [CLASSES.menu_item_icon].as_slice())
        );
        assert_eq!(tree.text(*label), "Copy path");
        assert_eq!(
            (tree.text(*shortcut), tree.classes(*shortcut)),
            ("⌥⌘C", [CLASSES.menu_item_shortcut].as_slice())
        );
        let [_, chevron] = tree.children(menu.lines[2]) else {
            panic!("a line with a submenu holds its label and a chevron");
        };
        assert_eq!(tree.classes(*chevron), [CLASSES.menu_item_chevron]);
        assert!(tree.states(menu.lines[3]).contains(States::DISABLED));
        assert_eq!(tree.classes(menu.lines[4]), [CLASSES.menu_separator]);
        assert_eq!(tree.classes(menu.lines[5]), [CLASSES.menu_item, CLASSES.danger]);

        let geometry = menu
            .stage
            .surfaces
            .overlay_geometry(menu.overlay)
            .expect("the anchor has a box after the frame");
        assert_eq!(geometry.anchor, menu.stage.tree().border_box(menu.trigger));
        assert_eq!(geometry.placement.side, OverlaySide::Below);
        assert_eq!(
            menu.stage.accessible_node_on(menu.overlay, panel).role(),
            AccessibleRole::Menu
        );
        let line = menu.stage.accessible_node_on(menu.overlay, menu.lines[1]);
        assert_eq!(
            (line.role(), line.label()),
            (AccessibleRole::MenuItem, Some("Copy path"))
        );
        assert!(menu.stage.accessible_node_on(menu.overlay, menu.lines[3]).is_disabled());
    }

    #[test]
    fn the_arrow_keys_move_the_highlight_past_what_cannot_be_chosen_and_return_chooses() {
        let OpenMenu {
            mut stage,
            trigger,
            is_open,
            chosen,
            overlay,
            lines,
        } = open_menu();
        let focused = |stage: &Stage| stage.tree_of(overlay).focused();
        assert_eq!(
            focused(&stage),
            Some(panel_of(&stage, overlay)),
            "an opened menu has the keyboard and highlights nothing"
        );

        for expected in [0, 1, 2, 5, 0] {
            assert!(press(&mut stage, overlay, Key::ArrowDown));
            assert_eq!(focused(&stage), Some(lines[expected]));
        }
        assert!(press(&mut stage, overlay, Key::ArrowUp));
        assert_eq!(focused(&stage), Some(lines[5]), "the highlight goes around the ends");
        assert!(press(&mut stage, overlay, Key::Home));
        assert_eq!(focused(&stage), Some(lines[0]));
        assert!(press(&mut stage, overlay, Key::End));
        assert_eq!(focused(&stage), Some(lines[5]));
        assert!(press(&mut stage, overlay, Key::Tab), "Tab is kept in the menu");
        assert_eq!(focused(&stage), Some(lines[5]));

        assert!(press(&mut stage, overlay, Key::Enter));

        assert_eq!(stage.runtime.get(chosen), ["move to trash"]);
        assert!(!stage.runtime.get(is_open), "choosing a command closes the menu");
        assert!(stage.overlays_of(stage.surface).is_empty());
        assert_eq!(stage.tree().focused(), Some(trigger));
    }

    #[test]
    fn the_right_arrow_opens_a_submenu_beside_its_line_and_the_left_arrow_closes_it() {
        let OpenMenu {
            mut stage,
            is_open,
            chosen,
            overlay,
            lines,
            ..
        } = open_menu();
        for _ in 0..3 {
            press(&mut stage, overlay, Key::ArrowDown);
        }

        assert!(press(&mut stage, overlay, Key::ArrowRight));

        let (submenu, submenu_lines) = submenu_of(&stage, overlay);
        let geometry = stage
            .surfaces
            .overlay_geometry(submenu)
            .expect("the line has a box after the frame");
        assert_eq!(geometry.anchor, stage.tree_of(overlay).border_box(lines[2]));
        assert_eq!(geometry.placement.side, OverlaySide::Right);
        assert_eq!(
            stage.tree_of(submenu).focused(),
            Some(submenu_lines[0]),
            "a submenu the keyboard opened highlights its first line"
        );

        assert!(press(&mut stage, submenu, Key::ArrowLeft));
        assert!(
            stage.overlays_of(overlay).is_empty(),
            "the left arrow closes the submenu"
        );
        assert!(stage.runtime.get(is_open), "and leaves the menu open");
        assert_eq!(stage.tree_of(overlay).focused(), Some(lines[2]));

        press(&mut stage, overlay, Key::ArrowRight);
        let (submenu, submenu_lines) = submenu_of(&stage, overlay);
        press(&mut stage, submenu, Key::ArrowDown);
        assert_eq!(stage.tree_of(submenu).focused(), Some(submenu_lines[1]));
        assert!(press(&mut stage, submenu, Key::Enter));

        assert_eq!(stage.runtime.get(chosen), ["textedit"]);
        assert!(!stage.runtime.get(is_open), "a command of a submenu closes every menu");
        assert!(stage.overlays_of(stage.surface).is_empty());
        assert!(!stage.surfaces.contains(submenu));
    }

    #[test]
    fn escape_closes_the_innermost_menu() {
        let OpenMenu {
            mut stage,
            is_open,
            overlay,
            ..
        } = open_menu();
        for _ in 0..3 {
            press(&mut stage, overlay, Key::ArrowDown);
        }
        press(&mut stage, overlay, Key::ArrowRight);
        let (submenu, _) = submenu_of(&stage, overlay);

        assert!(press(&mut stage, submenu, Key::Escape));
        assert!(stage.overlays_of(overlay).is_empty());
        assert!(stage.runtime.get(is_open));

        assert!(press(&mut stage, overlay, Key::Escape));
        assert!(!stage.runtime.get(is_open));
        assert!(stage.overlays_of(stage.surface).is_empty());
    }

    #[test]
    fn the_pointer_highlights_the_line_it_is_on_and_a_click_chooses() {
        let OpenMenu {
            mut stage,
            is_open,
            chosen,
            overlay,
            lines,
            ..
        } = open_menu();
        let panel = panel_of(&stage, overlay);

        hover(&mut stage, overlay, lines[1]);
        assert_eq!(stage.tree_of(overlay).focused(), Some(lines[1]));

        hover(&mut stage, overlay, lines[2]);
        let (submenu, submenu_lines) = submenu_of(&stage, overlay);
        assert_eq!(
            stage.tree_of(submenu).focused(),
            Some(panel_of(&stage, submenu)),
            "a submenu the pointer opened highlights nothing"
        );

        hover(&mut stage, overlay, lines[3]);
        assert!(
            stage.overlays_of(overlay).is_empty(),
            "the submenu closes when the pointer goes to another line"
        );
        assert_eq!(
            stage.tree_of(overlay).focused(),
            Some(panel),
            "a disabled line is not highlighted"
        );

        hover(&mut stage, overlay, lines[2]);
        let (submenu, submenu_lines_again) = submenu_of(&stage, overlay);
        assert_eq!(submenu_lines.len(), submenu_lines_again.len());
        let preview = stage.middle_on(submenu, submenu_lines_again[0]);
        stage.click_on(submenu, PointerButton::Primary, preview);
        stage.frame_all();

        assert_eq!(stage.runtime.get(chosen), ["preview"]);
        assert!(!stage.runtime.get(is_open));
        assert!(stage.overlays_of(stage.surface).is_empty());
    }

    #[test]
    fn a_press_outside_closes_a_menu_with_its_submenu() {
        let OpenMenu {
            mut stage,
            is_open,
            overlay,
            lines,
            ..
        } = open_menu();
        let elsewhere = stage.mount(|mount| Button::new("Elsewhere").mount(mount));
        stage.frame_all();
        hover(&mut stage, overlay, lines[2]);
        let (submenu, _) = submenu_of(&stage, overlay);

        click(&mut stage, elsewhere);

        assert!(!stage.runtime.get(is_open));
        assert!(stage.overlays_of(stage.surface).is_empty());
        assert!(!stage.surfaces.contains(overlay) && !stage.surfaces.contains(submenu));
    }

    #[test]
    fn a_click_on_a_disabled_line_chooses_nothing_and_leaves_the_menu_open() {
        let OpenMenu {
            mut stage,
            is_open,
            chosen,
            overlay,
            lines,
            ..
        } = open_menu();

        let disabled = stage.middle_on(overlay, lines[3]);
        stage.click_on(overlay, PointerButton::Primary, disabled);
        stage.frame_all();

        assert!(stage.runtime.get(chosen).is_empty());
        assert!(stage.runtime.get(is_open));
    }
}

mod context_menu {
    use super::*;

    /// A button with a context menu around it; returns the box around the button.
    fn context_menu(is_custom: bool) -> (Stage, ElementId, Signal<Vec<&'static str>>) {
        let mut stage = Stage::new();
        let chosen = stage.runtime.signal(Vec::new());
        let area = stage.mount(move |mount| {
            let target = Button::new("Downloads").mount(mount);
            let menu = ContextMenu::new(menu_items(chosen)).label("File actions");
            if is_custom {
                menu.custom().mount(mount, target)
            } else {
                menu.mount(mount, target)
            }
        });
        stage.frame_all();
        (stage, area, chosen)
    }

    fn secondary_click(stage: &mut Stage, position: Point) {
        stage.click_on(stage.surface, PointerButton::Secondary, position);
        stage.frame_all();
    }

    #[test]
    fn a_secondary_press_opens_the_menu_the_kit_draws_where_the_pointer_is() {
        let (mut stage, _, chosen) = context_menu(true);
        let first = Point { x: 30.0, y: 10.0 };
        let second = Point { x: 120.0, y: 20.0 };

        secondary_click(&mut stage, first);

        let overlay = overlay_over(&stage, stage.surface);
        let geometry = stage
            .surfaces
            .overlay_geometry(overlay)
            .expect("a point is a place at once");
        assert_eq!(geometry.anchor, Rect::new(first.x, first.y, 0.0, 0.0));
        assert_eq!(
            (
                geometry.placement.side,
                geometry.placement.alignment,
                geometry.placement.gap
            ),
            (OverlaySide::Below, OverlayAlignment::Start, 0.0)
        );
        assert_eq!(
            stage.tree_of(overlay).classes(panel_of(&stage, overlay)),
            [CLASSES.menu]
        );

        secondary_click(&mut stage, second);

        let moved = overlay_over(&stage, stage.surface);
        assert!(
            !stage.surfaces.contains(overlay),
            "the menu opens anew where it is asked for next"
        );
        let geometry = stage
            .surfaces
            .overlay_geometry(moved)
            .expect("a point is a place at once");
        assert_eq!(geometry.anchor, Rect::new(second.x, second.y, 0.0, 0.0));

        press(&mut stage, moved, Key::ArrowDown);
        press(&mut stage, moved, Key::Enter);
        assert_eq!(stage.runtime.get(chosen), ["quick look"]);
        assert!(stage.overlays_of(stage.surface).is_empty());
    }

    #[test]
    fn a_primary_press_outside_closes_the_menu_the_kit_draws() {
        let (mut stage, area, _) = context_menu(true);
        secondary_click(&mut stage, Point { x: 30.0, y: 10.0 });
        assert_eq!(stage.overlays_of(stage.surface).len(), 1);

        click(&mut stage, area);

        assert!(
            stage.overlays_of(stage.surface).is_empty(),
            "a menu at a point keeps no press of the surface under it"
        );
    }

    #[test]
    fn by_default_a_secondary_press_asks_the_host_for_the_menu_of_the_system() {
        let (mut stage, _, chosen) = context_menu(false);
        let position = Point { x: 30.0, y: 10.0 };

        secondary_click(&mut stage, position);

        assert!(
            stage.overlays_of(stage.surface).is_empty(),
            "the kit draws nothing itself"
        );
        let menu = stage
            .surfaces
            .surface_mut(stage.surface)
            .take_native_menu()
            .expect("the surface asks for the menu of the system");
        assert_eq!(menu.position, position);
        let command = |id, label: &str, is_disabled, shortcut| NativeMenuItem::Command {
            id,
            label: label.to_owned(),
            is_disabled,
            shortcut,
        };
        assert_eq!(
            menu.items,
            [
                command(
                    0,
                    "Quick Look",
                    false,
                    Some(KeyChord {
                        key: Key::Space,
                        modifiers: Modifiers::NONE,
                    })
                ),
                command(
                    1,
                    "Copy path",
                    false,
                    Some(KeyChord {
                        key: Key::Character('c'),
                        modifiers: Modifiers::COMMAND.union(Modifiers::OPTION),
                    })
                ),
                NativeMenuItem::Submenu {
                    label: String::from("Open with"),
                    items: vec![command(2, "Preview", false, None), command(3, "TextEdit", false, None)],
                },
                command(4, "Remove download", true, None),
                NativeMenuItem::Separator,
                command(5, "Move to Trash", false, None),
            ]
        );
        assert!(
            stage.surfaces.surface_mut(stage.surface).take_native_menu().is_none(),
            "the menu is asked for once"
        );

        stage
            .surfaces
            .surface_mut(stage.surface)
            .native_menu_command_chosen(&mut stage.runtime, 3);

        assert_eq!(stage.runtime.get(chosen), ["textedit"]);
    }
}

mod tooltip {
    use super::*;

    /// A button with a tooltip; returns the box around the button.
    fn tooltip() -> (Stage, ElementId) {
        let mut stage = Stage::new();
        let area = stage.mount(|mount| {
            let target = Button::new("Recalculate").mount(mount);
            Tooltip::new("Recalculate sizes").mount(mount, target)
        });
        stage.frame_all();
        (stage, area)
    }

    #[test]
    fn a_tooltip_shows_once_the_pointer_has_rested_and_goes_when_it_leaves() {
        let (mut stage, area) = tooltip();
        let window = stage.surface;

        hover(&mut stage, window, area);
        stage.frame_all_after(Duration::from_millis(500));
        assert!(
            stage.overlays_of(stage.surface).is_empty(),
            "the pointer has not rested long enough"
        );
        stage.frame_all_after(Duration::from_millis(100));

        let overlay = overlay_over(&stage, stage.surface);
        let bubble = panel_of(&stage, overlay);
        let geometry = stage
            .surfaces
            .overlay_geometry(overlay)
            .expect("the target has a box after the frame");
        assert_eq!(
            geometry.input,
            OverlayInput::None,
            "the pointer and the keys pass a tooltip by"
        );
        assert_eq!(
            (geometry.placement.side, geometry.placement.alignment),
            (OverlaySide::Below, OverlayAlignment::Center)
        );
        assert_eq!(geometry.anchor, stage.tree().border_box(area));
        let tree = stage.tree_of(overlay);
        assert_eq!(tree.classes(bubble), [CLASSES.tooltip]);
        assert_eq!(tree.text(tree.children(bubble)[0]), "Recalculate sizes");
        let node = stage.accessible_node_on(overlay, bubble);
        assert_eq!(
            (node.role(), node.label()),
            (AccessibleRole::Tooltip, Some("Recalculate sizes"))
        );

        stage.pointer_on(stage.surface, PointerInputKind::Exited, Point::default());
        stage.frame_all();

        assert!(stage.overlays_of(stage.surface).is_empty());
    }

    #[test]
    fn a_press_hides_a_tooltip_and_a_pointer_that_left_early_shows_none() {
        let (mut stage, area) = tooltip();
        let window = stage.surface;

        hover(&mut stage, window, area);
        stage.frame_all_after(Duration::from_millis(300));
        stage.pointer_on(stage.surface, PointerInputKind::Exited, Point::default());
        stage.frame_all();
        stage.frame_all_after(Duration::from_millis(600));
        assert!(
            stage.overlays_of(stage.surface).is_empty(),
            "a pointer that left takes its waiting along"
        );

        hover(&mut stage, window, area);
        stage.frame_all_after(Duration::from_millis(600));
        assert_eq!(stage.overlays_of(stage.surface).len(), 1);

        click(&mut stage, area);

        assert!(stage.overlays_of(stage.surface).is_empty(), "a press hides the tooltip");
    }
}

mod dialog {
    use super::*;

    struct ModalStage {
        stage: Stage,
        opener: ElementId,
        holder: ElementId,
        is_open: Signal<bool>,
        answers: Signal<Vec<&'static str>>,
    }

    /// A button that opens a modal panel `build` makes for the signal: the button has focus
    /// and the panel is open.
    fn opened(build: impl FnOnce(&mut Mount<'_>, Signal<bool>, Signal<Vec<&'static str>>) -> ElementId) -> ModalStage {
        let mut stage = Stage::new();
        let is_open = stage.runtime.signal(false);
        let answers = stage.runtime.signal(Vec::new());
        let opener = stage.mount(move |mount| {
            Button::new("Delete…")
                .on_press(move |runtime| runtime.set(is_open, true))
                .mount(mount)
        });
        let holder = stage.mount(move |mount| build(mount, is_open, answers));
        stage.frame();
        stage.click(opener);
        stage.frame();
        ModalStage {
            stage,
            opener,
            holder,
            is_open,
            answers,
        }
    }

    fn confirmation() -> ModalStage {
        opened(|mount, is_open, answers| {
            Dialog::new(is_open, "Delete 3 items permanently?")
                .message("This cannot be undone.")
                .cancel_action("Cancel", move |runtime| {
                    runtime.update(answers, |answers| answers.push("cancel"));
                })
                .default_action("Delete", move |runtime| {
                    runtime.update(answers, |answers| answers.push("delete"));
                })
                .mount(mount)
        })
    }

    /// The panel on the layer the holder holds, and the two buttons of the panel.
    fn parts(modal: &ModalStage) -> (ElementId, ElementId, ElementId, ElementId) {
        let tree = modal.stage.tree();
        let [layer] = tree.children(modal.holder) else {
            panic!("an open panel has its layer in the tree");
        };
        let [panel] = tree.children(*layer) else {
            panic!("the layer holds the panel");
        };
        let actions = *tree.children(*panel).last().expect("a panel ends with its buttons");
        let [cancel, default] = tree.children(actions) else {
            panic!("the panel has its two buttons");
        };
        (*layer, *panel, *cancel, *default)
    }

    #[test]
    fn an_open_dialog_lies_on_a_scrim_that_covers_the_surface() {
        let mut modal = confirmation();
        let (layer, panel, cancel, default) = parts(&modal);
        let tree = modal.stage.tree();

        assert_eq!(tree.classes(layer), [CLASSES.dialog_layer]);
        assert_eq!(tree.border_box(layer), Rect::new(0.0, 0.0, 400.0, 300.0));
        assert_eq!(tree.classes(panel), [CLASSES.dialog]);
        let panel_box = tree.border_box(panel);
        assert_eq!(
            (panel_box.x, panel_box.width),
            (30.0, 340.0),
            "the panel lies in the middle"
        );
        let [title, message, _actions] = tree.children(panel) else {
            panic!("a dialog holds its title, its message and its buttons");
        };
        assert_eq!(
            (tree.text(*title), tree.classes(*title)),
            ("Delete 3 items permanently?", [CLASSES.dialog_title].as_slice())
        );
        assert_eq!(tree.text(*message), "This cannot be undone.");
        assert_eq!(tree.classes(cancel), [CLASSES.button]);
        assert_eq!(
            tree.classes(default),
            [CLASSES.button, CLASSES.primary],
            "the default answer stands out"
        );
        let node = modal.stage.accessible_node(panel);
        assert_eq!(
            (node.role(), node.label()),
            (AccessibleRole::Dialog, Some("Delete 3 items permanently?"))
        );
    }

    #[test]
    fn focus_stays_inside_an_open_dialog() {
        let mut modal = confirmation();
        let (_, _, cancel, default) = parts(&modal);

        assert_eq!(
            modal.stage.tree().focused(),
            Some(default),
            "a dialog opens with focus on its default answer"
        );
        assert!(modal.stage.press_key(Key::Tab, Modifiers::NONE));
        assert_eq!(modal.stage.tree().focused(), Some(cancel));
        assert!(modal.stage.press_key(Key::Tab, Modifiers::NONE));
        assert_eq!(modal.stage.tree().focused(), Some(default));
        assert!(modal.stage.press_key(Key::Tab, Modifiers::SHIFT));
        assert_eq!(modal.stage.tree().focused(), Some(cancel));
        assert!(
            !modal.stage.focus(modal.opener),
            "nothing outside the dialog takes focus while it is open"
        );

        modal.stage.frame();
        modal.stage.click(modal.opener);
        assert_eq!(
            modal.stage.tree().focused(),
            Some(cancel),
            "the scrim takes the presses meant for what it covers"
        );
        assert!(modal.stage.runtime.get(modal.answers).is_empty());
    }

    #[test]
    fn return_gives_the_default_answer_and_focus_goes_back_where_it_was() {
        let mut modal = confirmation();
        let (_, panel, _, _) = parts(&modal);
        assert!(modal.stage.focus(panel));

        assert!(modal.stage.press_key(Key::Enter, Modifiers::NONE));
        modal.stage.frame();

        assert_eq!(modal.stage.runtime.get(modal.answers), ["delete"]);
        assert!(!modal.stage.runtime.get(modal.is_open));
        assert!(
            modal.stage.tree().children(modal.holder).is_empty(),
            "a closed dialog leaves the tree"
        );
        assert_eq!(modal.stage.tree().focused(), Some(modal.opener));
    }

    #[test]
    fn escape_gives_the_cancelling_answer_and_a_button_answers_when_pressed() {
        let mut modal = confirmation();

        assert!(modal.stage.press_key(Key::Escape, Modifiers::NONE));
        modal.stage.frame();
        assert_eq!(modal.stage.runtime.get(modal.answers), ["cancel"]);
        assert!(!modal.stage.runtime.get(modal.is_open));
        assert_eq!(modal.stage.tree().focused(), Some(modal.opener));

        modal.stage.click(modal.opener);
        modal.stage.frame();
        let (_, _, cancel, default) = parts(&modal);
        modal.stage.click(default);
        modal.stage.frame();
        assert_eq!(modal.stage.runtime.get(modal.answers), ["cancel", "delete"]);
        assert!(!modal.stage.runtime.get(modal.is_open));

        modal.stage.click(modal.opener);
        modal.stage.frame();
        let (_, _, cancel_again, _) = parts(&modal);
        assert_ne!(cancel, cancel_again, "a dialog is built anew every time it opens");
        assert!(modal.stage.focus(cancel_again));
        assert!(modal.stage.press_key(Key::Enter, Modifiers::NONE));
        modal.stage.frame();
        assert_eq!(
            modal.stage.runtime.get(modal.answers),
            ["cancel", "delete", "cancel"],
            "Return on a focused button presses that button"
        );
    }

    #[test]
    fn a_sheet_hangs_from_the_top_of_the_surface_and_escape_closes_it() {
        let mut modal = opened(|mount, is_open, answers| {
            Sheet::new(is_open, "Save changes?")
                .default_action("Save", move |runtime| {
                    runtime.update(answers, |answers| answers.push("save"));
                })
                .mount(mount)
        });
        let tree = modal.stage.tree();
        let [layer] = tree.children(modal.holder) else {
            panic!("an open sheet has its layer in the tree");
        };
        let [sheet] = tree.children(*layer) else {
            panic!("the layer holds the sheet");
        };
        let (layer, sheet) = (*layer, *sheet);

        assert_eq!(tree.classes(layer), [CLASSES.sheet_layer]);
        assert_eq!(tree.border_box(layer), Rect::new(0.0, 0.0, 400.0, 300.0));
        assert_eq!(tree.classes(sheet), [CLASSES.sheet]);
        assert_eq!(
            tree.border_box(sheet).origin(),
            Point::default(),
            "a sheet hangs from the top edge"
        );
        let save = tree.children(*tree.children(sheet).last().expect("a sheet ends with its buttons"))[0];
        assert_eq!(tree.focused(), Some(save));

        assert!(
            modal.stage.press_key(Key::Escape, Modifiers::NONE),
            "Escape closes a sheet that has no cancelling answer"
        );
        modal.stage.frame();

        assert!(modal.stage.runtime.get(modal.answers).is_empty());
        assert!(!modal.stage.runtime.get(modal.is_open));
        assert!(modal.stage.tree().children(modal.holder).is_empty());
        assert_eq!(modal.stage.tree().focused(), Some(modal.opener));
    }
}

mod toasts {
    use super::*;

    fn toast(id: u64, title: &str, detail: Option<&str>, tone: ToastTone) -> Toast {
        Toast {
            id,
            title: title.to_owned(),
            detail: detail.map(str::to_owned),
            tone,
        }
    }

    #[test]
    fn a_toast_shows_in_the_stack_and_leaves_once_its_lifetime_has_passed() {
        let mut stage = Stage::new();
        let toasts = stage.runtime.signal(Vec::new());
        let stack = stage.mount(move |mount| ToastStack::new(toasts).lifetime(Duration::from_secs(2)).mount(mount));
        stage.frame_all();
        assert_eq!(stage.tree().classes(stack), [CLASSES.toasts]);

        stage.runtime.update(toasts, |toasts| {
            toasts.push(toast(1, "Freed 42.1 GB", Some("Moved to Trash"), ToastTone::Success));
        });
        stage.frame_all();

        let [shown] = stage.tree().children(stack) else {
            panic!("the stack shows the toast");
        };
        let shown = *shown;
        let tree = stage.tree();
        assert_eq!(tree.classes(shown), [CLASSES.toast]);
        let [dot, lines] = tree.children(shown) else {
            panic!("a toast holds its dot and its lines");
        };
        assert_eq!(tree.classes(*dot), [CLASSES.toast_dot]);
        let [title, detail] = tree.children(*lines) else {
            panic!("the toast has two lines");
        };
        assert_eq!(tree.text(*title), "Freed 42.1 GB");
        assert_eq!(
            (tree.text(*detail), tree.classes(*detail)),
            ("Moved to Trash", [CLASSES.toast_detail].as_slice())
        );
        let node = stage.accessible_node(shown);
        assert_eq!(
            (node.role(), node.label()),
            (AccessibleRole::Alert, Some("Freed 42.1 GB. Moved to Trash"))
        );

        stage.frame_all_after(Duration::from_millis(1000));
        stage.runtime.update(toasts, |toasts| {
            toasts.push(toast(2, "Could not empty the Trash", None, ToastTone::Danger));
        });
        stage.frame_all();
        let [_, failed] = stage.tree().children(stack) else {
            panic!("the stack shows both toasts, the newest last");
        };
        let failed_dot = stage.tree().children(*failed)[0];
        assert_eq!(stage.tree().classes(failed_dot), [CLASSES.toast_dot, CLASSES.danger]);

        stage.frame_all_after(Duration::from_millis(1000));

        assert_eq!(
            stage.runtime.get(toasts),
            [toast(2, "Could not empty the Trash", None, ToastTone::Danger)],
            "the first toast left the signal by itself"
        );
        assert_eq!(stage.tree().children(stack).len(), 1);

        stage.frame_all_after(Duration::from_millis(1100));

        assert!(stage.runtime.get(toasts).is_empty());
        assert!(stage.tree().children(stack).is_empty());
    }
}
