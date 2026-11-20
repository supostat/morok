use std::fmt::Write;

use morok::components::view::Mount;
use morok::components::{CellAlignment, RowSelection, Table, TableColumn};
use morok::core::ElementId;

use crate::page_layout::{filled_page, filled_section};

/// The rows of the table: as many as a large folder of a large volume holds.
pub const TABLE_ROWS: usize = 1_000_000;
const KINDS: [&str; 4] = ["folder", "document", "disk image", "model"];
const WRITTEN: &str = "a string takes what is written to it";

/// A column of the table of documents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DocumentColumn {
    Name,
    Size,
    Modified,
    Kind,
}

/// `Table` with a million rows: columns the user resizes, moves and sorts by, rows selected
/// one or several at a time. Only the rows in view have elements, and scrolling allocates
/// nothing: a cell writes its text into the string it is given.
pub(crate) fn mount(mount: &mut Mount<'_>) -> ElementId {
    let column = |key, title: &str, width, alignment| TableColumn {
        key,
        title: title.to_owned(),
        width,
        alignment,
    };
    let columns = mount.runtime().signal(vec![
        column(DocumentColumn::Name, "Name", 300.0, CellAlignment::Start),
        column(DocumentColumn::Size, "Size", 110.0, CellAlignment::End),
        column(DocumentColumn::Modified, "Modified", 180.0, CellAlignment::Start),
        column(DocumentColumn::Kind, "Kind", 150.0, CellAlignment::Start),
    ]);
    let selected = mount.runtime().signal(vec![3]);
    let sort = mount.runtime().signal(None);
    let table = Table::new(
        TABLE_ROWS,
        columns,
        |_, row, column, text| {
            match column {
                DocumentColumn::Name => write!(text, "Document {}", row + 1),
                DocumentColumn::Size => write!(text, "{}.{} MB", row % 900 + 1, row % 10),
                DocumentColumn::Modified => write!(text, "Nov {}, {:02}:{:02}", row % 30 + 1, row % 24, row % 60),
                DocumentColumn::Kind => write!(text, "{}", KINDS[row % KINDS.len()]),
            }
            .expect(WRITTEN);
        },
        selected,
        sort,
    )
    .selection(RowSelection::Multiple)
    .label("Documents")
    .mount(mount);
    let section = filled_section(mount, "A MILLION ROWS", table);
    filled_page(mount, "Table", &[section])
}
