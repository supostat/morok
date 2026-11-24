use morok_components::view::Mount;
use morok_components::{
    Button, IconButton, KitIcon, SearchField, Sidebar, SidebarItem, SplitAxis, SplitPane, SplitView, StatusBar, Toolbar,
};
use morok_core::{ElementId, Key, KeyChord, Modifiers, Size};
use morok_css::SourceFile;
use morok_style::States;

use crate::case::Case;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;
use crate::suites::{component_stylesheet, kit_tokens};

pub(crate) const NAME: &str = "scaffold";

const COMPONENT_STYLESHEET_PATHS: [&str; 8] = [
    "crates/components/src/button.css",
    "crates/components/src/icon_button.css",
    "crates/components/src/search_field.css",
    "crates/components/src/sidebar.css",
    "crates/components/src/split_view.css",
    "crates/components/src/status_bar.css",
    "crates/components/src/text_field.css",
    "crates/components/src/toolbar.css",
];
const STYLESHEET_PATH: &str = "crates/teststand/suites/scaffold.css";
const STYLESHEET: &str = include_str!("../../suites/scaffold.css");

/// The states the tree gives a control the pointer presses.
const PRESSED: States = States::HOVER.union(States::ACTIVE);
/// The states the tree gives an element the keyboard focused.
const KEYBOARD_FOCUS: States = States::FOCUS.union(States::FOCUS_VISIBLE);

/// `Toolbar`, `StatusBar`, `Sidebar` and `SplitView` in their states, and the four put
/// together as the scaffolding of a window.
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
            case("toolbar", 640.0, 76.0, toolbar_case),
            case("status-bars", 640.0, 96.0, status_bars),
            case("sidebars", 428.0, 228.0, sidebars),
            case("split-view", 560.0, 180.0, split_view),
            case("split-vertical", 360.0, 240.0, split_vertical),
            case("window", 640.0, 320.0, window),
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

fn function_key(number: u8) -> KeyChord {
    KeyChord {
        key: Key::Function(number),
        modifiers: Modifiers::NONE,
    }
}

fn toolbar(mount: &mut Mount<'_>) -> ElementId {
    let query = mount.runtime().signal(String::new());
    Toolbar::new()
        .item(|mount| IconButton::new(KitIcon::Sidebar, "Sidebar").mount(mount))
        .title("Quartz Commander")
        .spacer()
        .item(move |mount| SearchField::new(query).placeholder("Filter").mount(mount))
        .item(|mount| Button::new("Clean up").primary().mount(mount))
        .label("Window")
        .mount(mount)
}

fn status_bar(mount: &mut Mount<'_>) -> ElementId {
    StatusBar::new()
        .text("Selected: 1 folder")
        .command(function_key(5), "Copy", |_| {})
        .command(function_key(6), "Move", |_| {})
        .command(function_key(8), "Delete", |_| {})
        .spacer()
        .text("Tab — other pane")
        .label("Status")
        .mount(mount)
}

fn sidebar(selected: &'static str) -> impl FnOnce(&mut Mount<'_>) -> ElementId {
    move |mount| {
        let place = mount.runtime().signal(selected);
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
            .item(
                SidebarItem::new("caches", "Build caches")
                    .icon(KitIcon::Bolt)
                    .detail("60 GB"),
            )
            .item(SidebarItem::new("duplicates", "Duplicates").icon(KitIcon::Copy))
            .label("Places")
            .mount(mount)
    }
}

/// A pane of the stand: a line of text with room around it.
fn pane(text: &'static str) -> impl FnOnce(&mut Mount<'_>) -> ElementId {
    move |mount| {
        let line = mount.text(text).id();
        let class = mount
            .tree()
            .style_sheet()
            .class_named("scaffold-pane")
            .expect("the stylesheet of the suite names the class of a pane");
        mount.element().class(class).child(line).id()
    }
}

/// A toolbar at the top of the surface.
fn toolbar_case(scene: &mut Scene) {
    let stage = scene.add(scene.root(), &["scaffold-bars"]);
    scene.mount(stage, toolbar);
}

/// A status bar at rest, and one with a command under the pointer, a pressed one and one
/// with the focus the keyboard gave.
fn status_bars(scene: &mut Scene) {
    let stage = scene.add(scene.root(), &["scaffold-bars"]);
    scene.mount(stage, status_bar);
    scene.mount(stage, status_bar);
    let commands = scene.elements_with_class("status-command");
    scene.set_state(commands[3], States::HOVER);
    scene.set_state(commands[4], PRESSED);
    scene.set_state(commands[5], KEYBOARD_FOCUS);
}

/// A sidebar with its first place selected, and one with another place selected and the
/// focus the keyboard gave on it.
fn sidebars(scene: &mut Scene) {
    let stage = scene.add(scene.root(), &["scaffold-sidebars"]);
    scene.mount(stage, sidebar("macintosh"));
    scene.mount(stage, sidebar("caches"));
    let items = scene.elements_with_class("sidebar-item");
    scene.set_state(items[6], KEYBOARD_FOCUS);
}

/// Three panes: the divider of the first under the pointer, with its handle, and the divider
/// of the last with the focus the keyboard gave.
fn split_view(scene: &mut Scene) {
    let stage = scene.add(scene.root(), &["scaffold-fill"]);
    scene.mount(stage, |mount| {
        let places_width = mount.runtime().signal(150.0_f32);
        let details_width = mount.runtime().signal(170.0_f32);
        SplitView::new()
            .pane(SplitPane::sized(places_width, 100.0..=240.0, pane("Places")))
            .pane(SplitPane::flexible(pane("Files")))
            .pane(SplitPane::sized(details_width, 120.0..=260.0, pane("Details")))
            .label("Panes")
            .mount(mount)
    });
    let grips = scene.elements_with_class("split-grip");
    scene.set_state(grips[0], States::HOVER);
    let dividers = scene.elements_with_class("split-divider");
    scene.set_state(dividers[1], KEYBOARD_FOCUS);
}

/// Three panes one above another: the divider of the first under the pointer, with its
/// handle, and the divider of the last with the focus the keyboard gave.
fn split_vertical(scene: &mut Scene) {
    let stage = scene.add(scene.root(), &["scaffold-fill"]);
    scene.mount(stage, |mount| {
        let places_height = mount.runtime().signal(60.0_f32);
        let details_height = mount.runtime().signal(70.0_f32);
        SplitView::new()
            .axis(SplitAxis::Vertical)
            .pane(SplitPane::sized(places_height, 40.0..=120.0, pane("Places")))
            .pane(SplitPane::flexible(pane("Files")))
            .pane(SplitPane::sized(details_height, 50.0..=140.0, pane("Details")))
            .label("Panes")
            .mount(mount)
    });
    let grips = scene.elements_with_class("split-grip");
    scene.set_state(grips[0], States::HOVER);
    let dividers = scene.elements_with_class("split-divider");
    scene.set_state(dividers[1], KEYBOARD_FOCUS);
}

/// The scaffolding of a window: a toolbar over a split view of a sidebar and the content,
/// over a status bar.
fn window(scene: &mut Scene) {
    let window = scene.add(scene.root(), &["scaffold-window"]);
    scene.mount(window, toolbar);
    let body = scene.add(window, &["scaffold-body"]);
    scene.mount(body, |mount| {
        let sidebar_width = mount.runtime().signal(190.0_f32);
        SplitView::new()
            .pane(SplitPane::sized(sidebar_width, 140.0..=280.0, sidebar("macintosh")))
            .pane(SplitPane::flexible(pane("12 items")))
            .label("Sidebar")
            .mount(mount)
    });
    scene.mount(window, status_bar);
}
