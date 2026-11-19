use morok_components::{
    Button, Dialog, KitIcon, Menu, MenuItem, Popover, Sheet, Toast, ToastStack, ToastTone, Tooltip,
};
use morok_core::{ElementId, Key, KeyChord, Modifiers, OverlaySide, Size};
use morok_css::SourceFile;
use morok_style::States;

use crate::case::Case;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;
use crate::suites::component_stylesheet;

pub(crate) const NAME: &str = "overlays";

const COMPONENT_STYLESHEET_PATHS: [&str; 7] = [
    "crates/components/src/button.css",
    "crates/components/src/dialog.css",
    "crates/components/src/menu.css",
    "crates/components/src/popover.css",
    "crates/components/src/sheet.css",
    "crates/components/src/toast_stack.css",
    "crates/components/src/tooltip.css",
];
const STYLESHEET_PATH: &str = "crates/teststand/suites/overlays.css";
const STYLESHEET: &str = include_str!("../../suites/overlays.css");

/// What `Menu`, `Tooltip` and `Popover` show in their overlays, laid out on the surface of
/// the case, and `ToastStack`, `Dialog` and `Sheet` on the surface they cover.
pub(crate) fn suite() -> Suite {
    Suite {
        name: NAME,
        component_stylesheets: COMPONENT_STYLESHEET_PATHS
            .iter()
            .map(|path| component_stylesheet(path))
            .collect(),
        stylesheets: vec![SourceFile::new(STYLESHEET_PATH, STYLESHEET)],
        cases: vec![
            case("menu", 300.0, 220.0, menu),
            case("tooltip", 260.0, 92.0, tooltip),
            case("popovers", 660.0, 220.0, popovers),
            case("toasts", 420.0, 200.0, toasts),
            case("dialog", 480.0, 300.0, dialog),
            case("sheet", 520.0, 260.0, sheet),
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
    scene.add(scene.root(), &["overlays-stage"])
}

/// A menu with a command, a highlighted command, a line that opens a submenu, a disabled
/// command and, past a separator, a destructive one.
fn menu(scene: &mut Scene) {
    let stage = stage(scene);
    scene.mount(stage, |mount| {
        let is_open = mount.runtime().signal(true);
        let items = vec![
            MenuItem::new("Quick Look").icon(KitIcon::Eye),
            MenuItem::new("Copy path").icon(KitIcon::Copy).shortcut(KeyChord {
                key: Key::Character('c'),
                modifiers: Modifiers::COMMAND.union(Modifiers::OPTION),
            }),
            MenuItem::submenu("Open with", vec![MenuItem::new("Preview")]).icon(KitIcon::Star),
            MenuItem::new("Remove download").icon(KitIcon::Cloud).disabled(true),
            MenuItem::separator(),
            MenuItem::new("Move to Trash")
                .icon(KitIcon::Trash)
                .shortcut(KeyChord {
                    key: Key::Backspace,
                    modifiers: Modifiers::COMMAND,
                })
                .destructive(),
        ];
        Menu::new(is_open, items).label("File actions").mount_panel(mount)
    });
    let highlighted = scene.elements_with_class("menu-item")[1];
    scene.set_state(highlighted, States::FOCUS);
}

fn tooltip(scene: &mut Scene) {
    let stage = stage(scene);
    scene.mount(stage, |mount| Tooltip::new("Recalculate sizes").mount_bubble(mount));
}

/// A popover below what it belongs to and one above it.
fn popovers(scene: &mut Scene) {
    let stage = stage(scene);
    for side in [OverlaySide::Below, OverlaySide::Above] {
        scene.mount(stage, |mount| {
            let is_open = mount.runtime().signal(true);
            Popover::new(is_open)
                .title("Derived data")
                .message("38.4 GB of derived data. It comes back with the next run — safe to delete.")
                .content(|mount| {
                    let clean_up = Button::new("Clean up").primary().mount(mount);
                    let later = Button::new("Later").mount(mount);
                    mount.element().child(clean_up).child(later).id()
                })
                .mount_panel(mount, side)
        });
    }
    for popover in scene.elements_with_class("popover") {
        let actions = *scene
            .tree()
            .children(popover)
            .last()
            .expect("a popover ends with its content");
        scene.add_class(actions, "overlays-actions");
    }
}

/// Two toasts in the corner of the surface: one of something done, one of something that failed.
fn toasts(scene: &mut Scene) {
    stage(scene);
    scene.mount(scene.root(), |mount| {
        let toasts = mount.runtime().signal(vec![
            Toast {
                id: 1,
                title: String::from("Freed 42.1 GB"),
                detail: Some(String::from("Moved to Trash")),
                tone: ToastTone::Success,
            },
            Toast {
                id: 2,
                title: String::from("Could not empty the Trash"),
                detail: None,
                tone: ToastTone::Danger,
            },
        ]);
        ToastStack::new(toasts).mount(mount)
    });
}

fn dialog(scene: &mut Scene) {
    stage(scene);
    scene.mount(scene.root(), |mount| {
        let is_open = mount.runtime().signal(true);
        Dialog::new(is_open, "Delete 3 items permanently?")
            .message("The three folders will be deleted without going to the Trash. This cannot be undone.")
            .cancel_action("Cancel", |_| {})
            .default_action("Delete", |_| {})
            .mount(mount)
    });
}

fn sheet(scene: &mut Scene) {
    stage(scene);
    scene.mount(scene.root(), |mount| {
        let is_open = mount.runtime().signal(true);
        Sheet::new(is_open, "Save changes to the layout?")
            .message("The columns and their order are kept for the next launch.")
            .cancel_action("Discard", |_| {})
            .default_action("Save", |_| {})
            .mount(mount)
    });
}
