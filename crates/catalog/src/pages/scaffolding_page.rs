use morok::components::view::{Mount, Surfaces};
use morok::components::{
    Button, IconButton, KitIcon, SearchField, Sidebar, SidebarItem, SplitAxis, SplitPane, SplitView, StatusBar, Toolbar,
};
use morok::core::{ElementId, Key, KeyChord, Modifiers};
use morok::reactive::Runtime;

use crate::page_layout::{page, readout, row, section};
use crate::styles::CLASSES;

/// A place the sidebar of the page selects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Place {
    Macintosh,
    Archive,
    Caches,
}

/// The scaffolding of a window, each part on its own: a toolbar and a status bar whose commands
/// are read back under them, a sidebar, a split view of three panes side by side whose last one
/// folds away, and a split view of two panes one above the other whose lower one folds away.
pub(crate) fn mount(mount: &mut Mount<'_>) -> ElementId {
    let query = mount.runtime().signal(String::new());
    let place = mount.runtime().signal(Place::Macintosh);
    let last_command = mount.runtime().signal(String::from("no command yet"));
    let said = mount
        .runtime()
        .memo(move |runtime| format!("last command: {}", runtime.get(last_command)));

    let toolbar = Toolbar::new()
        .item(|mount| IconButton::new(KitIcon::Sidebar, "Sidebar").mount(mount))
        .title("Quartz Commander")
        .spacer()
        .item(move |mount| {
            SearchField::new(query)
                .placeholder("Filter")
                .label("Filter")
                .mount(mount)
        })
        .item(move |mount| {
            Button::new("Clean up")
                .primary()
                .on_press(move |runtime| runtime.set(last_command, String::from("clean up")))
                .mount(mount)
        })
        .label("Window")
        .mount(mount);
    let command =
        |name: &'static str| move |runtime: &mut Runtime<Surfaces>| runtime.set(last_command, String::from(name));
    let status_bar = StatusBar::new()
        .text("Selected: 1 folder")
        .command(function_key(5), "Copy", command("copy"))
        .command(function_key(6), "Move", command("move"))
        .command(function_key(8), "Delete", command("delete"))
        .spacer()
        .text("Tab — other pane")
        .label("Status")
        .mount(mount);
    let bars = mount
        .element()
        .class(CLASSES.page_bars)
        .child(toolbar)
        .child(status_bar)
        .id();
    let said = readout(mount, said);
    let sidebar = Sidebar::new(place)
        .heading("VOLUMES")
        .item(
            SidebarItem::new(Place::Macintosh, "Macintosh HD")
                .icon(KitIcon::Disk)
                .detail("1 TB"),
        )
        .item(
            SidebarItem::new(Place::Archive, "Archive")
                .icon(KitIcon::Disk)
                .detail("4 TB"),
        )
        .heading("RECOMMENDATIONS")
        .item(
            SidebarItem::new(Place::Caches, "Build caches")
                .icon(KitIcon::Bolt)
                .detail("60 GB"),
        )
        .label("Places")
        .mount(mount);
    let sidebar_box = mount.element().class(CLASSES.page_sidebar).child(sidebar).id();
    let split_box = side_by_side_split(mount);
    let vertical_split_box = vertical_split(mount);
    let bars_section = section(mount, "TOOLBAR AND STATUS BAR", bars);
    let said_section = section(mount, "COMMANDS", said);
    let sidebar_row = row(mount, &[sidebar_box]);
    let sidebar_section = section(mount, "SIDEBAR", sidebar_row);
    let split_section = section(mount, "SPLIT VIEW", split_box);
    let vertical_split_section = section(mount, "VERTICAL SPLIT VIEW", vertical_split_box);
    page(
        mount,
        "Window scaffolding",
        &[
            bars_section,
            said_section,
            sidebar_section,
            split_section,
            vertical_split_section,
        ],
    )
}

/// Three panes side by side in a box of the page; the last one folds away.
fn side_by_side_split(mount: &mut Mount<'_>) -> ElementId {
    let places_width = mount.runtime().signal(150.0_f32);
    let details_width = mount.runtime().signal(170.0_f32);
    let are_details_folded = mount.runtime().signal(false);
    let split_view = SplitView::new()
        .pane(SplitPane::sized(places_width, 100.0..=240.0, pane("Places")))
        .pane(SplitPane::flexible(pane("Files")))
        .pane(SplitPane::sized(details_width, 120.0..=260.0, pane("Details")).collapsible(are_details_folded))
        .label("Panes")
        .mount(mount);
    mount.element().class(CLASSES.page_split).child(split_view).id()
}

/// Two panes one above the other in a box of the page; the lower one folds away.
fn vertical_split(mount: &mut Mount<'_>) -> ElementId {
    let preview_height = mount.runtime().signal(64.0_f32);
    let is_preview_folded = mount.runtime().signal(false);
    let split_view = SplitView::new()
        .axis(SplitAxis::Vertical)
        .pane(SplitPane::flexible(pane("Files")))
        .pane(SplitPane::sized(preview_height, 40.0..=110.0, pane("Preview")).collapsible(is_preview_folded))
        .label("Preview")
        .mount(mount);
    mount.element().class(CLASSES.page_split).child(split_view).id()
}

fn function_key(number: u8) -> KeyChord {
    KeyChord {
        key: Key::Function(number),
        modifiers: Modifiers::NONE,
    }
}

/// A pane of the split view: a line of text with room around it.
fn pane(text: &'static str) -> impl FnOnce(&mut Mount<'_>) -> ElementId {
    move |mount| {
        let line = mount.text(text).id();
        mount.element().class(CLASSES.page_pane).child(line).id()
    }
}
