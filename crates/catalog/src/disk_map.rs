use morok::components::view::Mount;
use morok::components::{
    Button, Dialog, KitIcon, ProgressBar, SegmentedControl, Sidebar, SidebarItem, StatusBar, Toolbar,
};
use morok::core::accesskit::Role;
use morok::core::{CanvasId, ElementId};
use morok::reactive::{Memo, Signal};

use crate::chart_view::ChartView;
use crate::disk_map_painter::DiskMapPainter;
use crate::styles::CLASSES;
use crate::volume::Volume;

/// The place the sidebar of the disk map selects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Place {
    Macintosh,
    Archive,
    BuildCaches,
    Duplicates,
}

impl Place {
    fn of(volume: Volume) -> Self {
        match volume {
            Volume::Macintosh => Self::Macintosh,
            Volume::Archive => Self::Archive,
        }
    }

    /// The volume the disk map shows while the place is selected: the recommendations are
    /// folders of Macintosh HD.
    fn volume(self) -> Volume {
        match self {
            Self::Macintosh | Self::BuildCaches | Self::Duplicates => Volume::Macintosh,
            Self::Archive => Volume::Archive,
        }
    }
}

/// A screen of a disk analyzer: the volumes and the recommendations in a sidebar, the folders of
/// the volume selected as a sunburst or a treemap on a canvas, the toolbar switching between the
/// two, and the folder the analysis picked out beside it. Cleaning up asks first.
#[derive(Clone, Copy, Debug)]
pub struct DiskMap {
    volume: Volume,
    view: ChartView,
}

impl DiskMap {
    /// The disk map opened on `volume` in `view`; the sidebar and the toolbar change both.
    pub fn showing(volume: Volume, view: ChartView) -> Self {
        Self { volume, view }
    }

    /// The canvas the chart of `volume` in `view` is drawn into: a canvas of its own for every
    /// chart, so that every window draws the chart it shows.
    pub fn canvas(volume: Volume, view: ChartView) -> CanvasId {
        match (volume, view) {
            (Volume::Macintosh, ChartView::Sunburst) => CanvasId(1),
            (Volume::Macintosh, ChartView::Treemap) => CanvasId(2),
            (Volume::Archive, ChartView::Sunburst) => CanvasId(3),
            (Volume::Archive, ChartView::Treemap) => CanvasId(4),
        }
    }

    /// What draws every chart into its canvas.
    pub fn painter() -> DiskMapPainter {
        DiskMapPainter::new()
    }

    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let is_asking_to_clean = mount.runtime().signal(false);
        let status = mount.runtime().signal(String::from("Scan finished in 4.1 s"));
        let place = mount.runtime().signal(Place::of(self.volume));
        let view = mount.runtime().signal(self.view);
        let volume = mount.runtime().memo(move |runtime| runtime.get(place).volume());

        let toolbar = Toolbar::new()
            .title("DOMOVOI")
            .spacer()
            .item(move |mount| {
                let mut views = SegmentedControl::new(view);
                for shown in ChartView::ALL {
                    views = views.segment(shown, shown.title());
                }
                views.label("Chart").mount(mount)
            })
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
                SidebarItem::new(Place::Macintosh, Volume::Macintosh.name())
                    .icon(KitIcon::Disk)
                    .detail(Volume::Macintosh.capacity()),
            )
            .item(
                SidebarItem::new(Place::Archive, Volume::Archive.name())
                    .icon(KitIcon::Disk)
                    .detail(Volume::Archive.capacity()),
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
        let chart = chart(mount, volume, view);
        let details = details(mount, volume);
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

/// The chart of the volume in the view on its canvas; the sunburst has the used space written
/// in its hole.
fn chart(mount: &mut Mount<'_>, volume: Memo<Volume>, view: Signal<ChartView>) -> ElementId {
    let shown = mount
        .runtime()
        .memo(move |runtime| DiskMap::canvas(runtime.get(volume), runtime.get(view)));
    let description = mount.runtime().memo(move |runtime| runtime.get(volume).description());
    let is_sunburst = mount
        .runtime()
        .memo(move |runtime| runtime.get(view) == ChartView::Sunburst);
    let canvas = mount
        .canvas(shown)
        .class(CLASSES.disk_map_canvas)
        .accessible_role(Role::Image)
        .accessible_name(description)
        .id();
    mount
        .element()
        .class(CLASSES.disk_map_chart)
        .child(canvas)
        .show(is_sunburst, move |mount| total(mount, volume))
        .id()
}

/// The used space of the volume and its capacity, written in the hole of the sunburst.
fn total(mount: &mut Mount<'_>, volume: Memo<Volume>) -> ElementId {
    let used = mount.runtime().memo(move |runtime| runtime.get(volume).used());
    let capacity = mount
        .runtime()
        .memo(move |runtime| format!("used of {}", runtime.get(volume).capacity()));
    let used = mount.text(used).class(CLASSES.disk_map_used).id();
    let capacity = mount.text(capacity).class(CLASSES.disk_map_capacity).id();
    mount
        .element()
        .class(CLASSES.disk_map_total)
        .child(used)
        .child(capacity)
        .id()
}

/// The folder the analysis of the volume picked out.
fn details(mount: &mut Mount<'_>, volume: Memo<Volume>) -> ElementId {
    let runtime = mount.runtime();
    let path = runtime.memo(move |runtime| runtime.get(volume).picked_folder().path.to_owned());
    let size = runtime.memo(move |runtime| runtime.get(volume).picked_folder().size.to_owned());
    let share = runtime.memo(move |runtime| runtime.get(volume).picked_folder().share);
    let note = runtime.memo(move |runtime| runtime.get(volume).picked_folder().note.to_owned());
    let label = mount.text("SELECTED").class(CLASSES.disk_map_label).id();
    let path = mount.text(path).class(CLASSES.disk_map_path).id();
    let size = mount.text(size).class(CLASSES.disk_map_detail).id();
    let share = ProgressBar::determinate(share)
        .label("Share of the used space")
        .mount(mount);
    let note = mount.text(note).class(CLASSES.disk_map_note).id();
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
