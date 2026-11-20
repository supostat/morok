use std::fmt::Write;

use morok::components::view::Mount;
use morok::components::{Button, Dialog, KitIcon, ProgressBar, Sidebar, SidebarItem, StatusBar, Toolbar};
use morok::core::accesskit::Role;
use morok::core::{CanvasId, ElementId};

use crate::folder_usage::FolderUsage;
use crate::styles::CLASSES;
use crate::sunburst::Sunburst;
use crate::sunburst_painter::SunburstPainter;
use crate::tokens::TOKENS;

/// The canvas the disk map draws its sunburst into.
pub const DISK_MAP_CANVAS: CanvasId = CanvasId(1);

/// The folders of the volume the disk map shows: 612 GB of a terabyte.
const FOLDERS: [FolderUsage; 6] = [
    FolderUsage {
        name: "Developer",
        gigabytes: 60.0,
        children: &[38.0, 22.0],
        color: TOKENS.color_chart_1,
    },
    FolderUsage {
        name: "Caches",
        gigabytes: 42.0,
        children: &[30.0, 8.0, 4.0],
        color: TOKENS.color_chart_2,
    },
    FolderUsage {
        name: "Photos",
        gigabytes: 180.0,
        children: &[90.0, 50.0, 40.0],
        color: TOKENS.color_chart_3,
    },
    FolderUsage {
        name: "Projects",
        gigabytes: 120.0,
        children: &[70.0, 30.0, 20.0],
        color: TOKENS.color_chart_4,
    },
    FolderUsage {
        name: "Music",
        gigabytes: 95.0,
        children: &[60.0, 35.0],
        color: TOKENS.color_chart_5,
    },
    FolderUsage {
        name: "System",
        gigabytes: 115.0,
        children: &[40.0, 40.0, 35.0],
        color: TOKENS.color_chart_6,
    },
];

/// The place the sidebar of the disk map selects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Place {
    Macintosh,
    Archive,
    BuildCaches,
    Duplicates,
}

/// A screen of a disk analyzer: the volumes and the recommendations in a sidebar, the folders of
/// the volume as a sunburst on a canvas with the used space written in its hole, and the folder
/// the analysis picked out beside it. Cleaning up asks first.
#[derive(Clone, Copy, Debug)]
pub struct DiskMap;

impl DiskMap {
    /// The sunburst of the folders of the volume.
    pub fn sunburst() -> Sunburst {
        Sunburst::new(&FOLDERS)
    }

    /// What draws the sunburst into [`DISK_MAP_CANVAS`].
    pub fn painter() -> SunburstPainter {
        SunburstPainter::new(DISK_MAP_CANVAS, Self::sunburst().sectors())
    }

    pub fn mount(mount: &mut Mount<'_>) -> ElementId {
        let is_asking_to_clean = mount.runtime().signal(false);
        let status = mount.runtime().signal(String::from("Scan finished in 4.1 s"));
        let place = mount.runtime().signal(Place::Macintosh);

        let toolbar = Toolbar::new()
            .title("DOMOVOI")
            .spacer()
            .item(move |mount| {
                Button::new("Clean up 66.2 GB")
                    .primary()
                    .on_press(move |runtime| runtime.set(is_asking_to_clean, true))
                    .mount(mount)
            })
            .label("Disk map")
            .mount(mount);
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
                SidebarItem::new(Place::BuildCaches, "Build caches")
                    .icon(KitIcon::Bolt)
                    .detail("60 GB"),
            )
            .item(
                SidebarItem::new(Place::Duplicates, "Duplicates")
                    .icon(KitIcon::Copy)
                    .detail("6 GB"),
            )
            .label("Volumes")
            .mount(mount);
        let chart = chart(mount);
        let details = details(mount);
        let body = mount
            .element()
            .class(CLASSES.disk_map_body)
            .child(sidebar)
            .child(chart)
            .child(details)
            .id();
        let status_bar = StatusBar::new()
            .text(status)
            .spacer()
            .text("model loaded · unloads in 5 min")
            .label("Status")
            .mount(mount);
        let dialog = Dialog::new(is_asking_to_clean, "Clean up 66.2 GB?")
            .message("Build caches and duplicates are removed. Xcode rebuilds its caches when it needs them.")
            .default_action("Clean Up", move |runtime| {
                runtime.set(status, String::from("Cleaned up 66.2 GB"));
            })
            .cancel_action("Cancel", |_| {})
            .mount(mount);
        mount
            .element()
            .class(CLASSES.disk_map)
            .child(toolbar)
            .child(body)
            .child(status_bar)
            .child(dialog)
            .id()
    }
}

/// The sunburst on its canvas, with the used space written in its hole.
fn chart(mount: &mut Mount<'_>) -> ElementId {
    let canvas = mount
        .canvas(DISK_MAP_CANVAS)
        .class(CLASSES.disk_map_canvas)
        .accessible_role(Role::Image)
        .accessible_name(chart_description())
        .id();
    let used = mount.text("612 GB").class(CLASSES.disk_map_used).id();
    let capacity = mount.text("used of 1 TB").class(CLASSES.disk_map_capacity).id();
    let total = mount
        .element()
        .class(CLASSES.disk_map_total)
        .child(used)
        .child(capacity)
        .id();
    mount
        .element()
        .class(CLASSES.disk_map_chart)
        .child(canvas)
        .child(total)
        .id()
}

/// What the sunburst shows, told in words: the used space, then every folder with its size.
fn chart_description() -> String {
    let mut description = String::from("Macintosh HD, 612 GB used of 1 TB:");
    for (position, folder) in FOLDERS.iter().enumerate() {
        let separator = if position == 0 { " " } else { ", " };
        write!(description, "{separator}{} {} GB", folder.name, folder.gigabytes)
            .expect("a string takes what is written to it");
    }
    description
}

/// The folder the analysis picked out.
fn details(mount: &mut Mount<'_>) -> ElementId {
    let label = mount.text("SELECTED").class(CLASSES.disk_map_label).id();
    let path = mount.text("~/Library/Developer").class(CLASSES.disk_map_path).id();
    let size = mount
        .text("60.1 GB · 214,038 files")
        .class(CLASSES.disk_map_detail)
        .id();
    let share = ProgressBar::determinate(0.38)
        .label("Share of the used space")
        .mount(mount);
    let note = mount
        .text("Local model: Xcode build caches and simulators, rebuilt automatically.")
        .class(CLASSES.disk_map_note)
        .id();
    mount
        .element()
        .class(CLASSES.disk_map_details)
        .child(label)
        .child(path)
        .child(size)
        .child(share)
        .child(note)
        .id()
}
