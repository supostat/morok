use std::fmt::Write;

use morok_components::view::Mount;
use morok_components::{CellAlignment, RowSelection, SortOrder, Table, TableColumn, TableSort};
use morok_core::{ElementId, Point, Size};
use morok_css::SourceFile;
use morok_style::States;

use crate::case::Case;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;
use crate::suites::component_stylesheet;

pub(crate) const NAME: &str = "table";

const COMPONENT_STYLESHEET_PATHS: [&str; 2] = [
    "crates/components/src/scroll_view.css",
    "crates/components/src/table.css",
];
const STYLESHEET_PATH: &str = "crates/teststand/suites/table.css";
const STYLESHEET: &str = include_str!("../../suites/table.css");

/// The states the tree gives an element the pointer presses.
const PRESSED: States = States::HOVER.union(States::ACTIVE);
/// The states the tree gives an element the keyboard focused.
const KEYBOARD_FOCUS: States = States::FOCUS.union(States::FOCUS_VISIBLE);

/// `Table` in every state: rows striped, selected, under the pointer and pressed; header cells
/// at rest, under the pointer, sorting the rows either way and dragged, with the edge that
/// resizes a column under the pointer; a table with focus; and a table scrolled both ways under
/// its header.
pub(crate) fn suite() -> Suite {
    Suite {
        name: NAME,
        component_stylesheets: COMPONENT_STYLESHEET_PATHS
            .iter()
            .map(|path| component_stylesheet(path))
            .collect(),
        stylesheets: vec![SourceFile::new(STYLESHEET_PATH, STYLESHEET)],
        cases: vec![
            case("table-states", 372.0, table_states),
            case("table-scrolled", 194.0, table_scrolled),
        ],
    }
}

fn case(name: &'static str, height: f32, build: fn(&mut Scene)) -> Case {
    Case {
        name,
        viewport: Size { width: 452.0, height },
        build,
        moment: Moment::AtRest,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileColumn {
    Name,
    Size,
    Modified,
    Kind,
}

const FILES: [(&str, &str, &str, &str); 5] = [
    ("DerivedData", "38.4 GB", "Today, 09:12", "folder"),
    ("CoreSimulator", "21.7 GB", "Yesterday", "folder"),
    ("llama-3.3-70b-q4.gguf", "39.6 GB", "Nov 2", "model"),
    ("node_modules", "6.1 GB", "Oct 28", "folder"),
    ("Ubuntu-24.04-arm64.iso", "2.9 GB", "Oct 4", "disk image"),
];

fn columns(widths: [f32; 4]) -> Vec<TableColumn<FileColumn>> {
    let column = |key, title: &str, width, alignment| TableColumn {
        key,
        title: title.to_owned(),
        width,
        alignment,
    };
    vec![
        column(FileColumn::Name, "Name", widths[0], CellAlignment::Start),
        column(FileColumn::Size, "Size", widths[1], CellAlignment::End),
        column(FileColumn::Modified, "Modified", widths[2], CellAlignment::Start),
        column(FileColumn::Kind, "Kind", widths[3], CellAlignment::Start),
    ]
}

/// A table of `row_count` files, the five files over and over, in columns of the widths.
fn files(
    row_count: usize,
    widths: [f32; 4],
    selected: Vec<usize>,
    sort: Option<TableSort<FileColumn>>,
) -> impl FnOnce(&mut Mount<'_>) -> ElementId {
    move |mount: &mut Mount<'_>| {
        let columns = mount.runtime().signal(columns(widths));
        let selected = mount.runtime().signal(selected);
        let sort = mount.runtime().signal(sort);
        Table::new(
            row_count,
            columns,
            |_, row, column, text| {
                let (name, size, modified, kind) = FILES[row % FILES.len()];
                let turn = row / FILES.len();
                match column {
                    FileColumn::Name if turn == 0 => text.push_str(name),
                    FileColumn::Name => {
                        write!(text, "{name} {}", turn + 1).expect("a string takes what is written to it");
                    }
                    FileColumn::Size => text.push_str(size),
                    FileColumn::Modified => text.push_str(modified),
                    FileColumn::Kind => text.push_str(kind),
                }
            },
            selected,
            sort,
        )
        .selection(RowSelection::Multiple)
        .mount(mount)
    }
}

/// The elements of a mounted table a case puts in states.
struct TableParts {
    viewport: ElementId,
    bars: [ElementId; 2],
    /// The row elements, by the slot.
    rows: Vec<ElementId>,
    /// The header cells, by the slot.
    header_cells: Vec<ElementId>,
}

impl TableParts {
    fn of(scene: &Scene, table: ElementId) -> Self {
        let tree = scene.tree();
        let [viewport, vertical_bar, horizontal_bar] = tree.children(table)[..] else {
            unreachable!("a table holds its viewport and two bars");
        };
        let content = tree.children(viewport)[0];
        let [rows, header] = tree.children(content)[..] else {
            unreachable!("the content of a table is its rows and its header");
        };
        Self {
            viewport,
            bars: [vertical_bar, horizontal_bar],
            rows: tree.children(rows).iter().map(|slot| tree.children(*slot)[0]).collect(),
            header_cells: tree.children(header).to_vec(),
        }
    }
}

/// A table sorted by its numbers, with a selected row, a row under the pointer and a pressed
/// one, a header cell under the pointer and the edge of another; and a table with focus, two
/// rows selected, sorted the other way by its names, a header cell dragged.
fn table_states(scene: &mut Scene) {
    let stage = scene.add(scene.root(), &["table-stage"]);
    let widths = [160.0, 80.0, 100.0, 70.0];
    let by_size = TableSort {
        column: FileColumn::Size,
        order: SortOrder::Descending,
    };
    let pointed = scene.mount(stage, files(5, widths, vec![1], Some(by_size)));
    let parts = TableParts::of(scene, pointed);
    scene.set_state(parts.rows[2], States::HOVER);
    scene.set_state(parts.rows[3], PRESSED);
    scene.set_state(parts.header_cells[0], States::HOVER);
    let edge = scene.tree().children(parts.header_cells[2])[2];
    scene.set_state(edge, States::HOVER);

    let by_name = TableSort {
        column: FileColumn::Name,
        order: SortOrder::Ascending,
    };
    let focused = scene.mount(stage, files(5, widths, vec![0, 2], Some(by_name)));
    scene.set_state(focused, States::FOCUS_WITHIN);
    let parts = TableParts::of(scene, focused);
    scene.set_state(parts.viewport, KEYBOARD_FOCUS);
    scene.set_state(parts.header_cells[1], PRESSED);
    scene.add_class(parts.header_cells[3], "dragged");
}

/// A table of more rows and wider columns than its viewport shows, scrolled down between two
/// rows and across into its first column, its bars under the pointer: the header stays at the
/// top and moves across with the columns.
fn table_scrolled(scene: &mut Scene) {
    let stage = scene.add(scene.root(), &["table-stage"]);
    let table = scene.mount(stage, files(24, [220.0, 90.0, 140.0, 110.0], vec![4], None));
    let parts = TableParts::of(scene, table);
    for bar in parts.bars {
        scene.set_state(bar, States::HOVER);
    }
    scene.scroll_to(parts.viewport, Point { x: 75.0, y: 70.0 });
}
