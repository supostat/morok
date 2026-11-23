use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{
    AccessibilityAction, AccessibleActions, AccessibleItem, ElementId, Focusability, Modifiers, Offset,
    PointerEventKind, Propagation,
};
use morok_reactive::{Memo, Runtime, Signal};
use morok_style::{Declaration, States};

use crate::column_edges::ColumnEdges;
use crate::column_slot::ColumnSlot;
use crate::length_in_pixels::pixels;
use crate::row_choice::RowChoice;
use crate::row_selection::RowSelection;
use crate::scroll_frame::ScrollFrame;
use crate::styles::CLASSES;
use crate::table_column::TableColumn;
use crate::table_columns::TableColumns;
use crate::table_header::{HEADER_HEIGHT, TableHeader};
use crate::table_sort::TableSort;
use crate::view::{ElementEvent, Mount, Property, Surfaces};
use crate::virtual_rows::VirtualRows;

/// How high a row of a table is.
const ROW_HEIGHT: f32 = 26.0;

/// Writes the text of a cell: of the row of the given index, in the column of the given key.
type CellText<ColumnKey> = dyn Fn(&mut Runtime<Surfaces>, usize, &ColumnKey, &mut String);

/// Rows of cells in columns under a header, as many rows and columns as there may be: only the
/// rows and the columns the viewport shows have elements, and a scroll allocates nothing.
///
/// The user drags the right edge of a header to make its column wider or narrower, drags a
/// header across the others to move its column, and clicks a header to sort the rows by its
/// column, in the other order on the next click. The table shows the columns and the sort it
/// is given signals of and writes what the user makes of them back; the application puts its
/// rows in the order the sort asks for. Rows are selected as in a list: the arrow keys move
/// through them, Home and End to the ends, and the table scrolls to keep the row the keyboard
/// is on in view.
pub struct Table<ColumnKey> {
    row_count: Property<usize>,
    columns: Signal<Vec<TableColumn<ColumnKey>>>,
    cell: Rc<CellText<ColumnKey>>,
    selected: Signal<Vec<usize>>,
    selection: RowSelection,
    sort: Signal<Option<TableSort<ColumnKey>>>,
    label: Property<String>,
}

impl<ColumnKey: Clone + PartialEq + 'static> Table<ColumnKey> {
    /// A table of `row_count` rows in the columns `columns` holds, left to right. `cell` writes
    /// the text of a cell, given the index of its row and the key of its column; a cell shows
    /// it anew whenever a signal `cell` read changes. The selected rows are the indices
    /// `selected` holds, in ascending order, and one row is selected at once. `sort` names the
    /// column the rows are sorted by.
    pub fn new(
        row_count: impl Into<Property<usize>>,
        columns: Signal<Vec<TableColumn<ColumnKey>>>,
        cell: impl Fn(&mut Runtime<Surfaces>, usize, &ColumnKey, &mut String) + 'static,
        selected: Signal<Vec<usize>>,
        sort: Signal<Option<TableSort<ColumnKey>>>,
    ) -> Self {
        Self {
            row_count: row_count.into(),
            columns,
            cell: Rc::new(cell),
            selected,
            selection: RowSelection::Single,
            sort,
            label: Property::Fixed(String::new()),
        }
    }

    #[must_use]
    pub fn selection(mut self, selection: RowSelection) -> Self {
        self.selection = selection;
        self
    }

    /// What an assistive technology calls the table.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// Builds the table: a view whose content scrolls, the header staying at its top, with a
    /// row slot for each row in view and in every one a cell slot for each column in view.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self {
            row_count,
            columns,
            cell,
            selected,
            selection,
            sort,
            label,
        } = self;
        let rows = mount.element().id();
        let header = mount.element().id();
        let content = mount
            .element()
            .class(CLASSES.table_content)
            .child(rows)
            .child(header)
            .id();
        let frame = ScrollFrame::mount(mount, content);
        let virtual_rows = VirtualRows::of_frame(mount, frame, ROW_HEIGHT, HEADER_HEIGHT);
        let runtime = mount.runtime();
        let row_count = runtime.memo(move |runtime| row_count.get(runtime));
        let table_columns = TableColumns::new(runtime, columns, virtual_rows.area);
        let choice = RowChoice::new(runtime, selected, selection);
        let content_width = runtime.memo(move |runtime| {
            Declaration::Width(pixels(runtime.with(table_columns.edges, ColumnEdges::total_width)))
        });
        mount.build(content).inline(content_width);
        TableHeader {
            columns: table_columns,
            sort,
            drag: mount.runtime().signal(None),
        }
        .mount(mount, header);
        virtual_rows.fill(mount, rows, row_count.into(), move |mount, index| {
            row(mount, index, choice, table_columns, &cell)
        });
        let size = mount.runtime().memo(move |runtime| AccessibleItem {
            row_count: Some(runtime.get(row_count)),
            column_count: Some(runtime.with(table_columns.edges, ColumnEdges::column_count)),
            ..AccessibleItem::default()
        });
        mount
            .build(frame.viewport)
            .focusable(Focusability::TabStop)
            .accessible_role(AccessibleRole::Table)
            .accessible_name(label)
            .accessible_item(size)
            .on_event(move |runtime, event| match event {
                ElementEvent::Key(key) => {
                    let row_count = runtime.get(row_count);
                    choice.key(runtime, key, row_count)
                }
                _ => Propagation::Continue,
            });
        virtual_rows.keep_in_view(mount, choice.keyboard_row);
        mount.build(frame.view).class(CLASSES.table);
        frame.view
    }
}

impl<ColumnKey: Debug> Debug for Table<ColumnKey> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("Table")
            .field("row_count", &self.row_count)
            .field("columns", &self.columns)
            .field("selected", &self.selected)
            .field("selection", &self.selection)
            .field("sort", &self.sort)
            .field("label", &self.label)
            .finish_non_exhaustive()
    }
}

/// Builds what a row slot holds: the row of the index, with a cell slot for each column in view.
fn row<ColumnKey: Clone + PartialEq + 'static>(
    mount: &mut Mount<'_>,
    index: Memo<usize>,
    choice: RowChoice,
    columns: TableColumns<ColumnKey>,
    cell: &Rc<CellText<ColumnKey>>,
) -> ElementId {
    let runtime = mount.runtime();
    let is_selected = choice.is_selected(runtime, index);
    let is_striped = runtime.memo(move |runtime| runtime.get(index) % 2 == 1);
    let place = runtime.memo(move |runtime| AccessibleItem {
        row_index: Some(runtime.get(index)),
        ..AccessibleItem::default()
    });
    let cell = Rc::clone(cell);
    mount
        .element()
        .class(CLASSES.table_row)
        .class_when(CLASSES.striped, is_striped)
        .state(States::CHECKED, is_selected)
        .accessible_role(AccessibleRole::Row)
        .accessible_item(place)
        .accessible_actions(AccessibleActions::CLICK)
        .on_event(move |runtime, event| {
            let index = runtime.get(index);
            match event {
                ElementEvent::Pointer(pointer) if matches!(pointer.kind, PointerEventKind::Clicked { .. }) => {
                    choice.click(runtime, index, pointer.modifiers);
                    Propagation::Stop
                }
                ElementEvent::Accessibility(AccessibilityAction::Click) => {
                    choice.click(runtime, index, Modifiers::NONE);
                    Propagation::Stop
                }
                _ => Propagation::Continue,
            }
        })
        .for_each(
            columns.slots,
            |slot| *slot,
            move |mount, slot| {
                let slot = columns.slot(mount.runtime(), *slot);
                body_cell(mount, index, slot, Rc::clone(&cell))
            },
        )
        .id()
}

/// Builds a cell slot of a row: a text as wide as the column it shows and moved to where that
/// column lies. Its box is of one size whatever it says, so a row that comes into view lays
/// out no box.
fn body_cell<ColumnKey: Clone + PartialEq + 'static>(
    mount: &mut Mount<'_>,
    row: Memo<usize>,
    slot: ColumnSlot<ColumnKey>,
    cell: Rc<CellText<ColumnKey>>,
) -> ElementId {
    let runtime = mount.runtime();
    let place = runtime.memo(move |runtime| AccessibleItem {
        column_index: Some(runtime.get(slot.column)),
        ..AccessibleItem::default()
    });
    let element = mount
        .text("")
        .class(CLASSES.table_cell)
        .class_when(CLASSES.end_aligned, slot.is_end_aligned)
        .inline(Declaration::Height(pixels(ROW_HEIGHT)))
        .inline(slot.width)
        .accessible_role(AccessibleRole::Cell)
        .accessible_item(place)
        .id();
    let mut text = String::new();
    mount.effect(move |runtime, tree| {
        let row = runtime.get(row);
        let key = runtime.get(slot.key);
        text.clear();
        cell(runtime, row, &key, &mut text);
        tree.set_text(element, &text);
    });
    mount.effect(move |runtime, tree| {
        let left = runtime.get(slot.left);
        tree.set_displacement(
            element,
            Offset {
                x: f64::from(left),
                y: 0.0,
            },
        );
    });
    element
}
