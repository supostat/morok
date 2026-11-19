use morok_reactive::{Memo, Runtime, Signal};
use morok_style::Declaration;

use crate::cell_alignment::CellAlignment;
use crate::column_edges::ColumnEdges;
use crate::column_slot::ColumnSlot;
use crate::length_in_pixels::pixels;
use crate::slot_window::SlotWindow;
use crate::table_column::TableColumn;
use crate::view::{ScrollArea, Surfaces};

/// The columns of a table as its header and its rows show them: where they lie, and which of
/// them the viewport shows. Only those have cell slots, a pool of them in the header and in
/// every row; a scroll across binds the slots of the columns that left the viewport to the
/// columns that came into it.
pub(crate) struct TableColumns<ColumnKey> {
    pub(crate) columns: Signal<Vec<TableColumn<ColumnKey>>>,
    pub(crate) edges: Memo<ColumnEdges>,
    /// The keys of the columns, left to right: what changes when a column moves, and not when
    /// one is made wider.
    keys: Memo<Vec<ColumnKey>>,
    window: Memo<SlotWindow>,
    /// The numbers of the cell slots.
    pub(crate) slots: Memo<Vec<usize>>,
}

impl<ColumnKey: Clone + PartialEq + 'static> TableColumns<ColumnKey> {
    /// The columns `columns` holds, in a viewport that `area` follows.
    pub(crate) fn new(
        runtime: &mut Runtime<Surfaces>,
        columns: Signal<Vec<TableColumn<ColumnKey>>>,
        area: Signal<ScrollArea>,
    ) -> Self {
        let edges = runtime.memo(move |runtime| {
            runtime.with(columns, |columns| {
                ColumnEdges::of_widths(columns.iter().map(|column| column.width))
            })
        });
        let keys = runtime.memo(move |runtime| {
            runtime.with(columns, |columns| {
                columns.iter().map(|column| column.key.clone()).collect::<Vec<_>>()
            })
        });
        let window = runtime.memo(move |runtime| {
            let area = runtime.get(area);
            runtime.with(edges, |edges| edges.window(area.offset.x, area.viewport.width))
        });
        let pool = runtime.memo(move |runtime| runtime.get(window).pool);
        let slots = runtime.memo(move |runtime| (0..runtime.get(pool)).collect::<Vec<usize>>());
        Self {
            columns,
            edges,
            keys,
            window,
            slots,
        }
    }

    /// What the cell slots of the number show.
    pub(crate) fn slot(self, runtime: &mut Runtime<Surfaces>, slot: usize) -> ColumnSlot<ColumnKey> {
        let Self {
            columns,
            edges,
            keys,
            window,
            ..
        } = self;
        let column = runtime.memo(move |runtime| runtime.get(window).item_of_slot(slot));
        let key = runtime.memo(move |runtime| {
            let column = runtime.get(column);
            runtime.with(keys, |keys| keys[column].clone())
        });
        let left = runtime.memo(move |runtime| {
            let column = runtime.get(column);
            runtime.with(edges, |edges| edges.left_of(column))
        });
        let width = runtime.memo(move |runtime| {
            let column = runtime.get(column);
            Declaration::Width(pixels(runtime.with(edges, |edges| edges.width_of(column))))
        });
        let is_end_aligned = runtime.memo(move |runtime| {
            let column = runtime.get(column);
            runtime.with(columns, |columns| columns[column].alignment == CellAlignment::End)
        });
        ColumnSlot {
            column,
            key,
            left,
            width,
            is_end_aligned,
        }
    }

    /// The place of the column of the key among the columns, zero for the leftmost; none when
    /// the table has no such column.
    pub(crate) fn place_of(self, runtime: &mut Runtime<Surfaces>, key: &ColumnKey) -> Option<usize> {
        runtime.with(self.keys, |keys| keys.iter().position(|held| held == key))
    }
}

impl<ColumnKey> Clone for TableColumns<ColumnKey> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<ColumnKey> Copy for TableColumns<ColumnKey> {}
