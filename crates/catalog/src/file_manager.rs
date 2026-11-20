use std::ops::RangeInclusive;

use morok::components::view::Mount;
use morok::components::{IconButton, KitIcon, SearchField, SplitPane, SplitView, StatusBar, Toolbar};
use morok::core::{ElementId, Key, KeyChord, Modifiers};
use morok::reactive::{Runtime, Signal};

use crate::file_entry::FileEntry;
use crate::file_pane::FilePane;
use crate::file_place::FilePlace;
use crate::pane_side::PaneSide;
use crate::styles::CLASSES;

const PROJECTS: FilePlace = FilePlace {
    path: "~/Projects",
    icon: KitIcon::Home,
    summary: "12 items",
    entries: &[
        FileEntry::WAY_UP,
        FileEntry::folder("morok", "Today"),
        FileEntry::folder("domovoi", "Yesterday"),
        FileEntry::folder("quartz-commander", "Nov 3"),
        FileEntry::folder("mneme-plugin", "Nov 8"),
        FileEntry::folder("facet", "Jun 24"),
        FileEntry::file("Cargo.toml", "1.2 KB", "Today"),
        FileEntry::file("Cargo.lock", "38 KB", "Today"),
        FileEntry::file("README.md", "4.8 KB", "Nov 2"),
        FileEntry::file("LICENSE-MIT", "1.1 KB", "Oct 9"),
        FileEntry::file("rustfmt.toml", "120 B", "Oct 9"),
        FileEntry::file("rust-toolchain.toml", "98 B", "Oct 9"),
        FileEntry::file("deny.toml", "2.3 KB", "Oct 12"),
    ],
};

const ARCHIVE: FilePlace = FilePlace {
    path: "/Volumes/Archive",
    icon: KitIcon::Disk,
    summary: "1.2 TB free",
    entries: &[
        FileEntry::WAY_UP,
        FileEntry::folder("Backups", "Nov 1"),
        FileEntry::folder("Photos 2025", "Jan 12"),
        FileEntry::file("models.tar.zst", "82 GB", "Oct 20"),
        FileEntry::file("cover.png", "2.1 MB", "Sep 9"),
    ],
};

/// How wide the second pane opens, and how far its divider moves.
const SECOND_PANE_WIDTH: f32 = 420.0;
const SECOND_PANE_LIMITS: RangeInclusive<f32> = 240.0..=640.0;
/// What the status bar says when no command was given yet.
const STATUS_HINT: &str = "Tab — other pane";

/// What a command of the status bar does to the selected entry of the active pane, put in
/// words for the status bar.
#[derive(Clone, Copy, Debug)]
enum Command {
    Copy,
    Move,
    NewFolder,
    Delete,
}

/// A screen of a two-pane file manager: the projects on the left and an archive volume on the
/// right, each a table of its entries under a header, a filter that narrows both, and the
/// commands of the function keys in the status bar. The button of the toolbar folds the second
/// pane away.
#[derive(Clone, Copy, Debug)]
pub struct FileManager;

impl FileManager {
    pub fn mount(mount: &mut Mount<'_>) -> ElementId {
        let filter = mount.runtime().signal(String::new());
        let active = mount.runtime().signal(PaneSide::Left);
        let is_second_pane_folded = mount.runtime().signal(false);
        let second_pane_width = mount.runtime().signal(SECOND_PANE_WIDTH);
        let last_command = mount.runtime().signal(None::<String>);
        let panes = [
            FilePane::new(mount, PaneSide::Left, &PROJECTS, filter),
            FilePane::new(mount, PaneSide::Right, &ARCHIVE, filter),
        ];

        let toolbar = Toolbar::new()
            .item(move |mount| {
                IconButton::new(KitIcon::Sidebar, "Fold the second pane")
                    .toggled(is_second_pane_folded)
                    .mount(mount)
            })
            .title("Quartz Commander")
            .spacer()
            .item(move |mount| {
                SearchField::new(filter)
                    .placeholder("Filter")
                    .label("Filter")
                    .mount(mount)
            })
            .label("Files")
            .mount(mount);
        let [left, right] = panes;
        let split_view = SplitView::new()
            .pane(SplitPane::flexible(move |mount| left.mount(mount, active)))
            .pane(
                SplitPane::sized(second_pane_width, SECOND_PANE_LIMITS, move |mount| {
                    right.mount(mount, active)
                })
                .collapsible(is_second_pane_folded),
            )
            .label("Panes")
            .mount(mount);
        let body = mount.element().class(CLASSES.file_manager_body).child(split_view).id();
        let selection = mount
            .runtime()
            .memo(move |runtime| selection_summary(runtime, &panes, active));
        let status = mount
            .runtime()
            .memo(move |runtime| runtime.get(last_command).unwrap_or_else(|| String::from(STATUS_HINT)));
        let command = move |command: Command| {
            move |runtime: &mut Runtime<_>| {
                let said = said_command(runtime, &panes, active, command);
                runtime.set(last_command, Some(said));
            }
        };
        let status_bar = StatusBar::new()
            .text(selection)
            .command(function_key(5), "Copy", command(Command::Copy))
            .command(function_key(6), "Move", command(Command::Move))
            .command(function_key(7), "Folder", command(Command::NewFolder))
            .command(function_key(8), "Delete", command(Command::Delete))
            .spacer()
            .text(status)
            .label("Status")
            .mount(mount);
        mount
            .element()
            .class(CLASSES.file_manager)
            .child(toolbar)
            .child(body)
            .child(status_bar)
            .id()
    }
}

fn function_key(number: u8) -> KeyChord {
    KeyChord {
        key: Key::Function(number),
        modifiers: Modifiers::NONE,
    }
}

fn pane_of(panes: &[FilePane; 2], side: PaneSide) -> &FilePane {
    match side {
        PaneSide::Left => &panes[0],
        PaneSide::Right => &panes[1],
    }
}

/// The entry selected in the active pane, when one is.
fn selected_entry<World>(
    runtime: &mut Runtime<World>,
    panes: &[FilePane; 2],
    active: Signal<PaneSide>,
) -> Option<FileEntry> {
    let pane = pane_of(panes, runtime.get(active));
    let place = pane.place;
    runtime.with(pane.selected, |selected| {
        selected.first().map(|index| place.entries[*index])
    })
}

fn selection_summary<World>(runtime: &mut Runtime<World>, panes: &[FilePane; 2], active: Signal<PaneSide>) -> String {
    match selected_entry(runtime, panes, active) {
        Some(entry) if entry.is_way_up() => String::from("Selected: the folder above"),
        Some(entry) if entry.is_folder() => String::from("Selected: 1 folder"),
        Some(_) => String::from("Selected: 1 file"),
        None => String::from("Nothing selected"),
    }
}

fn said_command<World>(
    runtime: &mut Runtime<World>,
    panes: &[FilePane; 2],
    active: Signal<PaneSide>,
    command: Command,
) -> String {
    let side = runtime.get(active);
    let here = pane_of(panes, side).place.path;
    let there = pane_of(panes, side.other()).place.path;
    let selected = selected_entry(runtime, panes, active)
        .filter(|entry| !entry.is_way_up())
        .map(|entry| entry.name);
    match (command, selected) {
        (Command::NewFolder, _) => format!("New folder in {here}"),
        (_, None) => String::from("Select an entry first"),
        (Command::Copy, Some(name)) => format!("Copy {name} to {there}"),
        (Command::Move, Some(name)) => format!("Move {name} to {there}"),
        (Command::Delete, Some(name)) => format!("Delete {name}"),
    }
}
