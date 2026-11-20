use morok::components::view::Mount;
use morok::components::{Combobox, Select};
use morok::core::ElementId;

use crate::page_layout::{column, page, readout, section};

/// A volume the select picks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Volume {
    Macintosh,
    Archive,
    TimeMachine,
}

/// `Select` and `Combobox`: a select of volumes, one disabled, and a combobox of hosts that
/// narrows its options to what is typed, one marked invalid.
pub(crate) fn mount(mount: &mut Mount<'_>) -> ElementId {
    let volume = mount.runtime().signal(Volume::Macintosh);
    let locked_volume = mount.runtime().signal(Volume::Archive);
    let host = mount.runtime().signal(String::new());
    let unknown_host = mount.runtime().signal(String::from("quartz.lan"));
    let picked = mount
        .runtime()
        .memo(move |runtime| format!("volume: {:?}, host: \"{}\"", runtime.get(volume), runtime.get(host)));

    let select = Select::new(volume)
        .option(Volume::Macintosh, "Macintosh HD")
        .option(Volume::Archive, "Archive")
        .option(Volume::TimeMachine, "Time Machine")
        .label("Volume")
        .mount(mount);
    let locked = Select::new(locked_volume)
        .option(Volume::Macintosh, "Macintosh HD")
        .option(Volume::Archive, "Archive")
        .label("Locked volume")
        .disabled(true)
        .mount(mount);
    let selects = column(mount, &[select, locked]);
    let combobox = Combobox::new(host)
        .option("quartz.local")
        .option("quartz-backup.local")
        .option("domovoi.local")
        .option("mneme.local")
        .placeholder("Host")
        .label("Host")
        .mount(mount);
    let invalid = Combobox::new(unknown_host)
        .option("quartz.local")
        .placeholder("Host")
        .label("Host that is not known")
        .invalid(true)
        .mount(mount);
    let comboboxes = column(mount, &[combobox, invalid]);
    let picked = readout(mount, picked);
    let select_section = section(mount, "SELECT", selects);
    let combobox_section = section(mount, "COMBOBOX", comboboxes);
    let picked_section = section(mount, "PICKED", picked);
    page(
        mount,
        "Select and combobox",
        &[select_section, combobox_section, picked_section],
    )
}
