use morok::components::view::{ElementEvent, Mount};
use morok::components::{CellAlignment, Table, TableColumn};
use morok::core::{ElementId, Propagation};
use morok::reactive::{Memo, Signal};

use crate::file_column::FileColumn;
use crate::file_entry::FileEntry;
use crate::file_place::FilePlace;
use crate::pane_side::PaneSide;
use crate::styles::CLASSES;

/// What the size column shows for a folder.
const FOLDER_SIZE: &str = "—";

/// A pane of the file manager: a header with the place it shows, over a table of the entries
/// whose names hold the filter. The pane the user pressed or typed in last is the active one.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FilePane {
    pub(crate) side: PaneSide,
    pub(crate) place: &'static FilePlace,
    /// The indices of the selected entries of the place.
    pub(crate) selected: Signal<Vec<usize>>,
    /// The indices of the entries the table shows, in their order.
    pub(crate) visible: Memo<Vec<usize>>,
}

impl FilePane {
    /// A pane of `place` whose table shows the entries `filter` lets through.
    pub(crate) fn new(
        mount: &mut Mount<'_>,
        side: PaneSide,
        place: &'static FilePlace,
        filter: Signal<String>,
    ) -> Self {
        let selected = mount.runtime().signal(vec![1]);
        let visible = mount.runtime().memo(move |runtime| {
            runtime.with(filter, |filter| {
                let filter = filter.to_lowercase();
                place
                    .entries
                    .iter()
                    .enumerate()
                    .filter(|(_, entry)| entry.is_way_up() || entry.name.to_lowercase().contains(&filter))
                    .map(|(index, _)| index)
                    .collect()
            })
        });
        Self {
            side,
            place,
            selected,
            visible,
        }
    }

    /// Builds the pane. A press or a key inside it makes it the active pane.
    pub(crate) fn mount(&self, mount: &mut Mount<'_>, active: Signal<PaneSide>) -> ElementId {
        let place = self.place;
        let visible = self.visible;
        let side = self.side;
        let icon = mount.kit_icon(place.icon).class(CLASSES.file_pane_icon).id();
        let path = mount.text(place.path).class(CLASSES.file_pane_path).id();
        let summary = mount.text(place.summary).class(CLASSES.file_pane_count).id();
        let header = mount
            .element()
            .class(CLASSES.file_pane_header)
            .child(icon)
            .child(path)
            .child(summary)
            .id();
        let columns = mount.runtime().signal(vec![
            column(FileColumn::Name, "Name", 220.0, CellAlignment::Start),
            column(FileColumn::Size, "Size", 70.0, CellAlignment::End),
            column(FileColumn::Modified, "Modified", 90.0, CellAlignment::Start),
        ]);
        let sort = mount.runtime().signal(None);
        let row_count = mount.runtime().memo(move |runtime| runtime.with(visible, Vec::len));
        let table = Table::new(
            row_count,
            columns,
            move |runtime, row, column, text| {
                let entry = runtime.with(visible, |visible| place.entries[visible[row]]);
                write_cell(&entry, *column, text);
            },
            self.selected,
            sort,
        )
        .label(place.path)
        .mount(mount);
        mount
            .element()
            .class(CLASSES.file_pane)
            .on_event(move |runtime, event| {
                let is_users = matches!(event, ElementEvent::Pointer(_) | ElementEvent::Key(_));
                if is_users && runtime.untracked(|runtime| runtime.get(active)) != side {
                    runtime.set(active, side);
                }
                Propagation::Continue
            })
            .child(header)
            .child(table)
            .id()
    }
}

fn column(key: FileColumn, title: &str, width: f32, alignment: CellAlignment) -> TableColumn<FileColumn> {
    TableColumn {
        key,
        title: title.to_owned(),
        width,
        alignment,
    }
}

fn write_cell(entry: &FileEntry, column: FileColumn, text: &mut String) {
    match column {
        FileColumn::Name => text.push_str(entry.name),
        FileColumn::Size => text.push_str(entry.size.unwrap_or(if entry.is_way_up() { "" } else { FOLDER_SIZE })),
        FileColumn::Modified => text.push_str(entry.modified),
    }
}
