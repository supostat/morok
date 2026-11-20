use morok::components::view::Mount;
use morok::components::{SearchField, TextArea, TextField};
use morok::core::ElementId;

use crate::page_layout::{column, page, readout, section};

/// `TextField`, `SearchField` and `TextArea`: empty with a placeholder, filled, invalid and
/// disabled, with what the first field holds read back under it.
pub(crate) fn mount(mount: &mut Mount<'_>) -> ElementId {
    let name = mount.runtime().signal(String::new());
    let host = mount.runtime().signal(String::from("quartz.local"));
    let port = mount.runtime().signal(String::from("80a"));
    let locked = mount.runtime().signal(String::from("morok"));
    let query = mount.runtime().signal(String::new());
    let notes = mount.runtime().signal(String::from(
        "Core Text shapes the text, the kit lays out the lines and draws the glyphs.",
    ));
    let typed = mount
        .runtime()
        .memo(move |runtime| format!("name: \"{}\"", runtime.get(name)));

    let name_field = TextField::new(name).placeholder("Name").label("Name").mount(mount);
    let typed = readout(mount, typed);
    let host_field = TextField::new(host).label("Host").mount(mount);
    let port_field = TextField::new(port).label("Port").invalid(true).mount(mount);
    let locked_field = TextField::new(locked).label("User").disabled(true).mount(mount);
    let fields = column(mount, &[name_field, typed, host_field, port_field, locked_field]);
    let search = SearchField::new(query)
        .placeholder("Search")
        .label("Search")
        .mount(mount);
    let searches = column(mount, &[search]);
    let notes_area = TextArea::new(notes).placeholder("Notes").label("Notes").mount(mount);
    let areas = column(mount, &[notes_area]);
    let fields_section = section(mount, "TEXT FIELD", fields);
    let search_section = section(mount, "SEARCH FIELD", searches);
    let area_section = section(mount, "TEXT AREA", areas);
    page(mount, "Text fields", &[fields_section, search_section, area_section])
}
