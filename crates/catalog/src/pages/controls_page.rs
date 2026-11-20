use morok::components::view::Mount;
use morok::components::{Checkbox, ProgressBar, RadioGroup, SegmentedControl, Slider, Spinner, Switch};
use morok::core::ElementId;

use crate::page_layout::{column, page, readout, row, section};

/// How a folder is sorted, for the radio group and the segmented control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arrangement {
    Name,
    Size,
    Date,
}

/// The controls that set a value: check boxes, a radio group, switches, a slider and a
/// segmented control, on and off and disabled, and the indicators of progress.
pub(crate) fn mount(mount: &mut Mount<'_>) -> ElementId {
    let shows_hidden = mount.runtime().signal(true);
    let follows_links = mount.runtime().signal(false);
    let is_locked = mount.runtime().signal(true);
    let arrangement = mount.runtime().signal(Arrangement::Name);
    let syncs = mount.runtime().signal(true);
    let compresses = mount.runtime().signal(false);
    let volume = mount.runtime().signal(40.0_f64);
    let view = mount.runtime().signal(Arrangement::Size);
    let volume_said = mount
        .runtime()
        .memo(move |runtime| format!("volume: {}", runtime.get(volume)));

    let hidden = Checkbox::new(shows_hidden, "Show hidden files").mount(mount);
    let links = Checkbox::new(follows_links, "Follow symbolic links").mount(mount);
    let locked = Checkbox::new(is_locked, "Locked by the system")
        .disabled(true)
        .mount(mount);
    let checkboxes = column(mount, &[hidden, links, locked]);
    let radios = RadioGroup::new(arrangement)
        .option(Arrangement::Name, "Name")
        .option(Arrangement::Size, "Size")
        .option(Arrangement::Date, "Date modified")
        .label("Sort by")
        .mount(mount);
    let sync = Switch::new(syncs, "Sync over iCloud").mount(mount);
    let compression = Switch::new(compresses, "Compress backups").mount(mount);
    let fixed = Switch::new(syncs, "Managed by the profile").disabled(true).mount(mount);
    let switches = column(mount, &[sync, compression, fixed]);
    let slider = Slider::new(volume).range(0.0..=100.0, 5.0).label("Volume").mount(mount);
    let volume_said = readout(mount, volume_said);
    let disabled_slider = Slider::new(volume).label("Balance").disabled(true).mount(mount);
    let sliders = column(mount, &[slider, volume_said, disabled_slider]);
    let segmented = SegmentedControl::new(view)
        .segment(Arrangement::Name, "List")
        .segment(Arrangement::Size, "Treemap")
        .segment(Arrangement::Date, "Timeline")
        .label("View")
        .mount(mount);
    let disabled_segmented = SegmentedControl::new(view)
        .segment(Arrangement::Name, "List")
        .segment(Arrangement::Size, "Treemap")
        .label("Locked view")
        .disabled(true)
        .mount(mount);
    let segments = column(mount, &[segmented, disabled_segmented]);
    let copying = ProgressBar::determinate(0.38).label("Copying").mount(mount);
    let scanning = ProgressBar::indeterminate().label("Scanning").mount(mount);
    let spinner = Spinner::new().label("Loading").mount(mount);
    let progress = column(mount, &[copying, scanning, spinner]);

    let choices = row(mount, &[checkboxes, radios]);
    let choices_section = section(mount, "CHECK BOX AND RADIO GROUP", choices);
    let switches_section = section(mount, "SWITCH", switches);
    let sliders_section = section(mount, "SLIDER", sliders);
    let segments_section = section(mount, "SEGMENTED CONTROL", segments);
    let progress_section = section(mount, "PROGRESS", progress);
    page(
        mount,
        "Controls",
        &[
            choices_section,
            switches_section,
            sliders_section,
            segments_section,
            progress_section,
        ],
    )
}
