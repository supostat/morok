mod support;

use std::borrow::Cow;
use std::fmt::Write;

use morok_components::view::Mount;
use morok_components::{CellAlignment, RowSelection, SortOrder, Table, TableColumn, TableSort};
use morok_core::accesskit::{Role as AccessibleRole, SortDirection};
use morok_core::{ElementId, Key, Modifiers, Offset, Point};
use morok_reactive::Signal;
use morok_style::{Declaration, LengthPercentage, LengthPercentageAuto, Pixels};
use support::Stage;

const ROW_HEIGHT: f32 = 26.0;
const HEADER_HEIGHT: f32 = 28.0;
/// The rows a table keeps from the start, and the columns.
const FIRST_ROW_POOL: usize = 32;
const FIRST_COLUMN_POOL: usize = 8;
/// The rows and the columns of the largest table, and how wide each column is.
const ROWS: usize = 1_000_000;
const COLUMNS: usize = 100;
const COLUMN_WIDTH: f32 = 80.0;

fn pixels(length: f32) -> LengthPercentageAuto {
    LengthPercentageAuto::LengthPercentage(LengthPercentage {
        pixels: Pixels(length),
        fraction: None,
        tokens: Cow::Borrowed(&[]),
    })
}

/// Gives a table 360 by 200 pixels: with its border, a viewport of 358 by 198.
fn sized(mount: &mut Mount<'_>, view: ElementId) -> ElementId {
    mount
        .build(view)
        .inline(Declaration::Width(pixels(360.0)))
        .inline(Declaration::Height(pixels(200.0)));
    view
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileColumn {
    Name,
    Size,
    Kind,
}

const FILE_COUNT: usize = 40;

/// A table of forty files in three columns. The application shows the files from the last to
/// the first while the rows are sorted in descending order.
struct Files {
    stage: Stage,
    table: TableParts,
    columns: Signal<Vec<TableColumn<FileColumn>>>,
    selected: Signal<Vec<usize>>,
    sort: Signal<Option<TableSort<FileColumn>>>,
}

impl Files {
    fn new() -> Self {
        let mut stage = Stage::new();
        let column = |key, title: &str, width, alignment| TableColumn {
            key,
            title: title.to_owned(),
            width,
            alignment,
        };
        let columns = stage.runtime.signal(vec![
            column(FileColumn::Name, "Name", 160.0, CellAlignment::Start),
            column(FileColumn::Size, "Size", 80.0, CellAlignment::End),
            column(FileColumn::Kind, "Kind", 100.0, CellAlignment::Start),
        ]);
        let selected = stage.runtime.signal(Vec::new());
        let sort = stage.runtime.signal(None);
        let view = stage.mount(move |mount| {
            let view = Table::new(
                FILE_COUNT,
                columns,
                move |runtime, row, column, text| {
                    let is_reversed = runtime.with(sort, |sort| {
                        sort.is_some_and(|sort: TableSort<FileColumn>| sort.order == SortOrder::Descending)
                    });
                    let file = if is_reversed { FILE_COUNT - 1 - row } else { row };
                    match column {
                        FileColumn::Name => write!(text, "File {file}"),
                        FileColumn::Size => write!(text, "{} KB", file * 3),
                        FileColumn::Kind => write!(text, "document"),
                    }
                    .expect("a string takes what is written to it");
                },
                selected,
                sort,
            )
            .selection(RowSelection::Multiple)
            .label("Files")
            .mount(mount);
            sized(mount, view)
        });
        stage.frame();
        let table = TableParts::of(&stage, view);
        Self {
            stage,
            table,
            columns,
            selected,
            sort,
        }
    }

    fn column_keys(&mut self) -> Vec<FileColumn> {
        self.stage.runtime.with(self.columns, |columns| {
            columns.iter().map(|column| column.key).collect()
        })
    }

    fn column_widths(&mut self) -> Vec<f32> {
        self.stage.runtime.with(self.columns, |columns| {
            columns.iter().map(|column| column.width).collect()
        })
    }
}

/// The elements a table is built from.
#[derive(Clone, Copy)]
struct TableParts {
    viewport: ElementId,
    content: ElementId,
    /// The element that holds the row slots.
    rows: ElementId,
    header: ElementId,
}

impl TableParts {
    fn of(stage: &Stage, view: ElementId) -> Self {
        let viewport = stage.children(view)[0];
        let content = stage.children(viewport)[0];
        let parts = stage.children(content);
        Self {
            viewport,
            content,
            rows: parts[0],
            header: parts[1],
        }
    }

    /// The row elements, by the slot.
    fn row_elements(self, stage: &Stage) -> Vec<ElementId> {
        stage
            .children(self.rows)
            .into_iter()
            .map(|slot| stage.children(slot)[0])
            .collect()
    }

    /// The row element whose slot the surface shows where the row of the index lies.
    fn row(self, stage: &Stage, index: usize) -> ElementId {
        let tree = stage.tree();
        let top = tree.surface_box(self.rows).y + index as f32 * ROW_HEIGHT;
        stage
            .children(self.rows)
            .into_iter()
            .find(|slot| (tree.surface_box(*slot).y - top).abs() < 0.5)
            .map_or_else(|| panic!("no slot shows row {index}"), |slot| stage.children(slot)[0])
    }

    /// The texts of the cells of a row, in the order the surface shows them left to right.
    fn texts_of_row(self, stage: &Stage, index: usize) -> Vec<String> {
        let tree = stage.tree();
        let mut cells = stage.children(self.row(stage, index));
        cells.sort_by(|first, second| tree.surface_box(*first).x.total_cmp(&tree.surface_box(*second).x));
        cells.into_iter().map(|cell| tree.text(cell).to_owned()).collect()
    }

    /// The header cells, in the order the surface shows them left to right.
    fn header_cells(self, stage: &Stage) -> Vec<ElementId> {
        let tree = stage.tree();
        let mut cells = stage.children(self.header);
        cells.sort_by(|first, second| tree.surface_box(*first).x.total_cmp(&tree.surface_box(*second).x));
        cells
    }

    fn element_count(self, stage: &Stage) -> usize {
        let tree = stage.tree();
        let mut count = 0;
        let mut pending = vec![self.viewport];
        while let Some(element) = pending.pop() {
            count += 1;
            pending.extend(tree.children(element));
        }
        count
    }

    /// The cell of a row in view that shows the text.
    fn cell_showing(self, stage: &Stage, text: &str) -> ElementId {
        let tree = stage.tree();
        self.row_elements(stage)
            .into_iter()
            .flat_map(|row| stage.children(row))
            .find(|cell| tree.text(*cell) == text)
            .unwrap_or_else(|| panic!("no cell shows {text}"))
    }

    /// The header cell of the title.
    fn header_cell_titled(self, stage: &Stage, title: &str) -> ElementId {
        let tree = stage.tree();
        self.header_cells(stage)
            .into_iter()
            .find(|cell| tree.text(stage.children(*cell)[0]) == title)
            .unwrap_or_else(|| panic!("no header cell is titled {title}"))
    }

    /// Scrolls the content to `offset` and runs the frames that show it: one that finds the
    /// limits and one whose slots follow the offset.
    fn scroll_to(self, stage: &mut Stage, offset: Offset) {
        stage
            .surfaces
            .surface_mut(stage.surface)
            .tree_mut()
            .scroll_to(self.viewport, offset);
        stage.frame();
        stage.frame();
    }
}

/// The edge of a header cell the user drags to resize its column.
fn resizer_of(stage: &Stage, header_cell: ElementId) -> ElementId {
    stage.children(header_cell)[2]
}

/// The left top corner of the box the surface shows the element in.
fn corner_of(stage: &Stage, element: ElementId) -> Point {
    let surface_box = stage.tree().surface_box(element);
    Point {
        x: surface_box.x,
        y: surface_box.y,
    }
}

/// A table of a million rows in a hundred columns, each cell showing its row and its column.
fn million_rows_in_a_hundred_columns() -> (Stage, TableParts) {
    let mut stage = Stage::new();
    let view = stage.mount(|mount| {
        let columns = (0..COLUMNS)
            .map(|key| TableColumn {
                key,
                title: format!("Column {key}"),
                width: COLUMN_WIDTH,
                alignment: CellAlignment::Start,
            })
            .collect();
        let columns = mount.runtime().signal(columns);
        let selected = mount.runtime().signal(Vec::new());
        let sort = mount.runtime().signal(None);
        let view = Table::new(
            ROWS,
            columns,
            |_, row, column, text| write!(text, "{row}:{column}").expect("a string takes what is written to it"),
            selected,
            sort,
        )
        .mount(mount);
        sized(mount, view)
    });
    stage.frame();
    stage.frame();
    let table = TableParts::of(&stage, view);
    (stage, table)
}

#[test]
fn a_million_rows_in_a_hundred_columns_have_cells_for_what_is_in_view_alone() {
    let (mut stage, table) = million_rows_in_a_hundred_columns();
    let rows = table.row_elements(&stage);
    let elements = table.element_count(&stage);

    assert_eq!(rows.len(), FIRST_ROW_POOL, "eight rows in view keep the first pool");
    assert!(
        rows.iter().all(|row| stage.children(*row).len() == FIRST_COLUMN_POOL),
        "five columns in view keep the first pool"
    );
    assert_eq!(stage.children(table.header).len(), FIRST_COLUMN_POOL);
    assert_eq!(table.texts_of_row(&stage, 0)[..3], ["0:0", "0:1", "0:2"]);
    assert_eq!(
        stage.tree().border_box(table.content).width.to_bits(),
        (COLUMNS as f32 * COLUMN_WIDTH).to_bits(),
        "the content is as wide as every column"
    );

    table.scroll_to(
        &mut stage,
        Offset {
            x: f64::from(50.0 * COLUMN_WIDTH),
            y: f64::from(500_000.0 * ROW_HEIGHT),
        },
    );

    assert_eq!(table.row_elements(&stage), rows, "every row slot stays");
    assert_eq!(table.element_count(&stage), elements);
    let texts = table.texts_of_row(&stage, 500_000);
    assert!(
        ["500000:50", "500000:51", "500000:54"]
            .iter()
            .all(|text| texts.contains(&(*text).to_owned())),
        "{texts:?}"
    );
    let tree = stage.tree();
    let shown = stage
        .children(table.row(&stage, 500_000))
        .into_iter()
        .find(|cell| tree.text(*cell) == "500000:50")
        .expect("the row shows the column scrolled to");
    let viewport_corner = corner_of(&stage, table.viewport);
    assert_eq!(
        corner_of(&stage, shown),
        Point {
            x: viewport_corner.x,
            y: viewport_corner.y + HEADER_HEIGHT
        },
        "the cell scrolled to begins at the left edge of the viewport, under the header"
    );
    let titles: Vec<String> = table
        .header_cells(&stage)
        .into_iter()
        .map(|cell| tree.text(stage.children(cell)[0]).to_owned())
        .collect();
    assert!(titles.contains(&String::from("Column 50")), "{titles:?}");
}

#[test]
fn far_down_a_million_rows_keep_their_cells_and_their_header_exactly_in_place() {
    let (mut stage, table) = million_rows_in_a_hundred_columns();
    let odd_far_down = 16_777_217.0;
    let scrolled = Offset {
        x: f64::from(50.0 * COLUMN_WIDTH),
        y: odd_far_down,
    };
    table.scroll_to(&mut stage, scrolled);
    assert_eq!(
        stage.tree().scroll_offset(table.viewport),
        scrolled,
        "2^24 + 1 down, which single precision rounds to 2^24"
    );

    // Row 645 278 begins 26 × 645 278 = 16 777 228 pixels down the rows: 11 below the offset.
    let first_whole_row: usize = 645_278;
    let viewport_corner = corner_of(&stage, table.viewport);
    for (place, row) in (first_whole_row..first_whole_row + 5).enumerate() {
        let cell = table.cell_showing(&stage, &format!("{row}:50"));
        assert_eq!(
            corner_of(&stage, cell),
            Point {
                x: viewport_corner.x,
                y: viewport_corner.y + HEADER_HEIGHT + 11.0 + place as f32 * ROW_HEIGHT,
            },
            "the cell of row {row} in the column scrolled to"
        );
    }
    assert_eq!(
        corner_of(&stage, table.header_cell_titled(&stage, "Column 50")),
        viewport_corner,
        "the header stays at the top of the viewport"
    );
}

#[test]
fn dragging_the_edge_of_a_header_resizes_its_column() {
    let mut files = Files::new();
    let name_header = files.table.header_cells(&files.stage)[0];
    let edge = files.stage.middle_of(resizer_of(&files.stage, name_header));

    files.stage.drag(
        edge,
        &[Point {
            x: edge.x + 40.0,
            y: edge.y,
        }],
    );
    assert_eq!(files.column_widths(), [200.0, 80.0, 100.0]);
    assert_eq!(
        files.stage.runtime.get(files.sort),
        None,
        "a drag of the edge sorts nothing"
    );
    files.stage.frame();
    let tree = files.stage.tree();
    let cells = files.stage.children(files.table.row(&files.stage, 0));
    let widths: Vec<f32> = cells.iter().map(|cell| tree.border_box(*cell).width).collect();
    assert_eq!(widths, [200.0, 80.0, 100.0]);
    let lefts: Vec<f32> = cells
        .iter()
        .map(|cell| tree.surface_box(*cell).x - tree.surface_box(files.table.content).x)
        .collect();
    assert_eq!(lefts, [0.0, 200.0, 280.0], "the columns after it make way");

    let name_header = files.table.header_cells(&files.stage)[0];
    let edge = files.stage.middle_of(resizer_of(&files.stage, name_header));
    files.stage.drag(
        edge,
        &[Point {
            x: edge.x - 500.0,
            y: edge.y,
        }],
    );
    assert_eq!(
        files.column_widths(),
        [40.0, 80.0, 100.0],
        "a column keeps its narrowest width"
    );
}

#[test]
fn a_click_on_a_header_sorts_the_rows_by_its_column_and_the_next_click_the_other_way() {
    let mut files = Files::new();
    let headers = files.table.header_cells(&files.stage);
    let size_header = headers[1];
    assert_eq!(files.stage.accessible_node(size_header).sort_direction(), None);

    files.stage.click(size_header);
    assert_eq!(
        files.stage.runtime.get(files.sort),
        Some(TableSort {
            column: FileColumn::Size,
            order: SortOrder::Ascending
        })
    );
    files.stage.frame();
    let node = files.stage.accessible_node(size_header);
    assert_eq!(node.role(), AccessibleRole::ColumnHeader);
    assert_eq!((node.label(), node.column_index()), (Some("Size"), Some(1)));
    assert_eq!(node.sort_direction(), Some(SortDirection::Ascending));
    assert_eq!(
        files.table.texts_of_row(&files.stage, 0),
        ["File 0", "0 KB", "document"]
    );

    files.stage.click(size_header);
    files.stage.frame();
    assert_eq!(
        files.stage.accessible_node(size_header).sort_direction(),
        Some(SortDirection::Descending)
    );
    assert_eq!(
        files.table.texts_of_row(&files.stage, 0),
        ["File 39", "117 KB", "document"],
        "the cells show the rows in the order the application puts them in"
    );

    files.stage.accessibility_click(headers[0]);
    assert_eq!(
        files.stage.runtime.get(files.sort),
        Some(TableSort {
            column: FileColumn::Name,
            order: SortOrder::Ascending
        }),
        "another column sorts in ascending order first"
    );
    files.stage.frame();
    assert_eq!(files.stage.accessible_node(size_header).sort_direction(), None);
}

#[test]
fn dragging_a_header_past_the_middle_of_the_next_one_moves_its_column() {
    let mut files = Files::new();
    let name_header = files.table.header_cells(&files.stage)[0];
    let pressed = files.stage.middle_of(name_header);
    let at = |travel: f32| Point {
        x: pressed.x + travel,
        y: pressed.y,
    };

    files.stage.drag(pressed, &[at(2.0)]);
    assert_eq!(
        files.column_keys(),
        [FileColumn::Name, FileColumn::Size, FileColumn::Kind],
        "a pointer that hardly moves drags nothing"
    );
    files.stage.runtime.set(files.sort, None);

    files.stage.drag(pressed, &[Point { x: pressed.x, y: 280.0 }]);
    files.stage.move_pointer(at(124.0));
    assert_eq!(
        files.column_keys(),
        [FileColumn::Name, FileColumn::Size, FileColumn::Kind],
        "with the button up the pointer drags nothing"
    );
    assert_eq!(
        files.stage.runtime.get(files.sort),
        None,
        "a press let go of off the header sorts nothing"
    );

    files.stage.drag(pressed, &[at(60.0), at(124.0)]);
    assert_eq!(
        files.column_keys(),
        [FileColumn::Size, FileColumn::Name, FileColumn::Kind],
        "the pointer passed the middle of the next column"
    );
    assert_eq!(
        files.stage.runtime.get(files.sort),
        None,
        "a drag of a header sorts nothing"
    );
    files.stage.frame();
    assert_eq!(
        files.table.texts_of_row(&files.stage, 1),
        ["3 KB", "File 1", "document"]
    );
    let tree = files.stage.tree();
    let titles: Vec<&str> = files
        .table
        .header_cells(&files.stage)
        .into_iter()
        .map(|cell| tree.text(files.stage.children(cell)[0]))
        .collect();
    assert_eq!(titles, ["Size", "Name", "Kind"]);

    let name_header = files.table.header_cells(&files.stage)[1];
    let pressed = files.stage.middle_of(name_header);
    files.stage.drag(
        pressed,
        &[Point {
            x: pressed.x - 130.0,
            y: pressed.y,
        }],
    );
    assert_eq!(
        files.column_keys(),
        [FileColumn::Name, FileColumn::Size, FileColumn::Kind],
        "and back past the middle of the column before it"
    );
}

#[test]
fn a_click_and_the_keys_select_rows_and_the_row_the_keyboard_is_on_stays_in_view() {
    let mut files = Files::new();
    let table = files.table;

    files.stage.click(table.row(&files.stage, 2));
    assert_eq!(files.stage.runtime.get(files.selected), [2]);
    assert!(files.stage.focus(table.viewport));
    assert!(files.stage.press_key(Key::ArrowDown, Modifiers::NONE));
    assert!(files.stage.press_key(Key::ArrowDown, Modifiers::NONE));
    assert_eq!(files.stage.runtime.get(files.selected), [4]);
    assert!(files.stage.press_key(Key::ArrowDown, Modifiers::SHIFT));
    assert!(files.stage.press_key(Key::ArrowDown, Modifiers::SHIFT));
    assert_eq!(files.stage.runtime.get(files.selected), [4, 5, 6]);
    files.stage.frame();
    files.stage.frame();
    assert_eq!(
        files.stage.tree().scroll_offset(table.viewport),
        Offset {
            x: 0.0,
            y: f64::from(7.0 * ROW_HEIGHT - (198.0 - HEADER_HEIGHT))
        },
        "the table scrolls as little as shows row 6 whole under the header"
    );

    assert!(files.stage.press_key(Key::End, Modifiers::NONE));
    assert_eq!(files.stage.runtime.get(files.selected), [FILE_COUNT - 1]);
    files.stage.frame();
    files.stage.frame();
    let tree = files.stage.tree();
    assert_eq!(tree.scroll_offset(table.viewport), tree.scroll_limit(table.viewport));
    assert_eq!(
        files.table.texts_of_row(&files.stage, FILE_COUNT - 1)[0],
        "File 39",
        "the last row has a slot"
    );

    assert!(files.stage.press_key(Key::Home, Modifiers::NONE));
    assert_eq!(files.stage.runtime.get(files.selected), [0]);
    files.stage.frame();
    files.stage.frame();
    assert_eq!(files.stage.tree().scroll_offset(table.viewport), Offset::default());
}

#[test]
fn an_assistive_technology_reads_a_table_of_rows_of_cells_under_column_headers() {
    let mut files = Files::new();
    let table = files.table;
    let second_row = table.row(&files.stage, 1);
    files.stage.accessibility_click(second_row);
    files.stage.frame();

    let node = files.stage.accessible_node(table.viewport);
    assert_eq!((node.role(), node.label()), (AccessibleRole::Table, Some("Files")));
    assert_eq!((node.row_count(), node.column_count()), (Some(FILE_COUNT), Some(3)));
    let row = files.stage.accessible_node(second_row);
    assert_eq!(row.role(), AccessibleRole::Row);
    assert_eq!((row.row_index(), row.is_selected()), (Some(1), Some(true)));
    let size_cell = files.stage.children(second_row)[1];
    let cell = files.stage.accessible_node(size_cell);
    assert_eq!(cell.role(), AccessibleRole::Cell);
    assert_eq!((cell.column_index(), cell.value()), (Some(1), Some("3 KB")));
    assert_eq!(files.stage.accessible_node(table.header).role(), AccessibleRole::Row);
    assert_eq!(files.stage.runtime.get(files.selected), [1]);
}
