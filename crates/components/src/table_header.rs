use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{
    AccessibilityAction, AccessibleActions, AccessibleItem, ElementId, Offset, PointerButton, PointerEvent,
    PointerEventKind, Propagation,
};
use morok_reactive::{Runtime, Signal};
use morok_style::Declaration;

use crate::column_drag::{ColumnDrag, DragPhase};
use crate::column_slot::ColumnSlot;
use crate::kit_icon::KitIcon;
use crate::length_in_pixels::pixels;
use crate::sort_order::SortOrder;
use crate::styles::CLASSES;
use crate::table_columns::TableColumns;
use crate::table_sort::TableSort;
use crate::view::{ElementEvent, Mount, Surfaces};

/// How high the header of a table is.
pub(crate) const HEADER_HEIGHT: f32 = 28.0;
/// The narrowest the user makes a column.
const NARROWEST_COLUMN: f32 = 40.0;
/// How far the pointer travels with the button down on a header before it drags the column.
const DRAG_SLOP: f32 = 4.0;

/// The header of a table: a cell slot for each column in view, with the title of the column,
/// the arrow of the order its rows are sorted in, and the edge the user drags to resize it. A
/// click on a cell sorts the rows by its column; a drag moves the column across the others,
/// which make way as the pointer passes their middles.
pub(crate) struct TableHeader<ColumnKey> {
    pub(crate) columns: TableColumns<ColumnKey>,
    pub(crate) sort: Signal<Option<TableSort<ColumnKey>>>,
    /// The press on a header cell that is under way, or that the button just came up from.
    pub(crate) drag: Signal<Option<ColumnDrag<ColumnKey>>>,
}

impl<ColumnKey: Clone + PartialEq + 'static> TableHeader<ColumnKey> {
    /// Fills `header` with the cell slots.
    pub(crate) fn mount(self, mount: &mut Mount<'_>, header: ElementId) {
        mount
            .build(header)
            .class(CLASSES.table_header)
            .inline(Declaration::Height(pixels(HEADER_HEIGHT)))
            .accessible_role(AccessibleRole::Row)
            .for_each(
                self.columns.slots,
                |slot| *slot,
                move |mount, slot| {
                    let slot = self.columns.slot(mount.runtime(), *slot);
                    self.cell(mount, slot)
                },
            );
    }

    fn cell(self, mount: &mut Mount<'_>, slot: ColumnSlot<ColumnKey>) -> ElementId {
        let Self { columns, sort, drag } = self;
        let runtime = mount.runtime();
        let title = runtime.memo(move |runtime| {
            let column = runtime.get(slot.column);
            runtime.with(columns.columns, |columns| columns[column].title.clone())
        });
        let order = runtime.memo(move |runtime| {
            let key = runtime.get(slot.key);
            runtime.with(sort, |sort| {
                sort.as_ref()
                    .filter(|sorted| sorted.column == key)
                    .map(|sorted| sorted.order)
            })
        });
        let is_sorted = runtime.memo(move |runtime| runtime.get(order).is_some());
        let is_ascending = runtime.memo(move |runtime| runtime.get(order) == Some(SortOrder::Ascending));
        let is_descending = runtime.memo(move |runtime| runtime.get(order) == Some(SortOrder::Descending));
        let is_dragged = runtime.memo(move |runtime| {
            let key = runtime.get(slot.key);
            runtime.with(drag, |drag| {
                drag.as_ref()
                    .is_some_and(|drag| drag.phase == DragPhase::Moving && drag.key == key)
            })
        });
        let place = runtime.memo(move |runtime| AccessibleItem {
            column_index: Some(runtime.get(slot.column)),
            sort_direction: runtime.get(order).map(Into::into),
            ..AccessibleItem::default()
        });
        let title_text = mount.text(title).id();
        let arrow = mount
            .kit_icon(KitIcon::Sort)
            .class(CLASSES.table_sort)
            .class_when(CLASSES.ascending, is_ascending)
            .class_when(CLASSES.descending, is_descending)
            .id();
        let resizer = self.resizer(mount, slot);
        let element = mount
            .element()
            .class(CLASSES.table_header_cell)
            .class_when(CLASSES.end_aligned, slot.is_end_aligned)
            .class_when(CLASSES.sorted, is_sorted)
            .class_when(CLASSES.dragged, is_dragged)
            .inline(slot.width)
            .accessible_role(AccessibleRole::ColumnHeader)
            .accessible_name(title)
            .accessible_item(place)
            .accessible_actions(AccessibleActions::CLICK)
            .on_event(move |runtime, event| match event {
                ElementEvent::Pointer(pointer) => self.pointer(runtime, slot, pointer),
                ElementEvent::Accessibility(AccessibilityAction::Click) => {
                    let key = runtime.get(slot.key);
                    self.sort_by(runtime, &key);
                    Propagation::Stop
                }
                _ => Propagation::Continue,
            })
            .child(title_text)
            .child(arrow)
            .child(resizer)
            .id();
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

    /// The right edge of a header cell: while the primary button is down on it, the column is
    /// as much wider as the pointer has travelled to the right.
    fn resizer(self, mount: &mut Mount<'_>, slot: ColumnSlot<ColumnKey>) -> ElementId {
        let columns = self.columns.columns;
        let mut resize: Option<ColumnResize<ColumnKey>> = None;
        mount
            .element()
            .class(CLASSES.table_resizer)
            .on_event(move |runtime, event| {
                let ElementEvent::Pointer(pointer) = event else {
                    return Propagation::Continue;
                };
                match pointer.kind {
                    PointerEventKind::Pressed {
                        button: PointerButton::Primary,
                        ..
                    } => {
                        let column = runtime.get(slot.column);
                        resize = Some(ColumnResize {
                            key: runtime.get(slot.key),
                            pressed_at: pointer.position.x,
                            width_at_press: runtime.with(columns, |columns| columns[column].width),
                        });
                        Propagation::Stop
                    }
                    PointerEventKind::Moved => {
                        let Some(resize) = &resize else {
                            return Propagation::Continue;
                        };
                        let width =
                            (resize.width_at_press + pointer.position.x - resize.pressed_at).max(NARROWEST_COLUMN);
                        runtime.update(columns, |columns| {
                            if let Some(column) = columns.iter_mut().find(|column| column.key == resize.key) {
                                column.width = width;
                            }
                        });
                        Propagation::Stop
                    }
                    PointerEventKind::Released {
                        button: PointerButton::Primary,
                    } => {
                        resize = None;
                        Propagation::Stop
                    }
                    PointerEventKind::Clicked { .. } => Propagation::Stop,
                    _ => Propagation::Continue,
                }
            })
            .id()
    }

    /// What the pointer does to a header cell.
    fn pointer(
        self,
        runtime: &mut Runtime<Surfaces>,
        slot: ColumnSlot<ColumnKey>,
        pointer: &PointerEvent,
    ) -> Propagation {
        match pointer.kind {
            PointerEventKind::Pressed {
                button: PointerButton::Primary,
                ..
            } => {
                let key = runtime.get(slot.key);
                let left = runtime.get(slot.left);
                let pressed = ColumnDrag {
                    key,
                    pressed_at: pointer.position.x,
                    pressed_in_content: left + pointer.position.x - pointer.element_box.x,
                    phase: DragPhase::Pressed,
                };
                runtime.set(self.drag, Some(pressed));
                Propagation::Stop
            }
            PointerEventKind::Moved => self.drag_to(runtime, pointer.position.x),
            PointerEventKind::Released {
                button: PointerButton::Primary,
            } => {
                runtime.update(self.drag, |drag| {
                    if let Some(drag) = drag {
                        drag.phase = drag.phase.released();
                    }
                });
                Propagation::Stop
            }
            PointerEventKind::Clicked { .. } => {
                let Some(pressed) = runtime.get(self.drag) else {
                    return Propagation::Continue;
                };
                runtime.set(self.drag, None);
                if pressed.phase == DragPhase::Released {
                    self.sort_by(runtime, &pressed.key);
                }
                Propagation::Stop
            }
            _ => Propagation::Continue,
        }
    }

    /// The pointer moved to `position` across the surface: once it has travelled far enough
    /// with the button down, the pressed column follows it, changing places with a column next
    /// to it whose middle the pointer has passed.
    fn drag_to(self, runtime: &mut Runtime<Surfaces>, position: f32) -> Propagation {
        let Some(mut drag) = runtime.get(self.drag) else {
            return Propagation::Continue;
        };
        let travelled = position - drag.pressed_at;
        match drag.phase {
            DragPhase::Released | DragPhase::Dropped => return Propagation::Continue,
            DragPhase::Pressed if travelled.abs() < DRAG_SLOP => return Propagation::Stop,
            DragPhase::Pressed => {
                drag.phase = DragPhase::Moving;
                runtime.set(self.drag, Some(drag.clone()));
            }
            DragPhase::Moving => {}
        }
        let Some(place) = self.columns.place_of(runtime, &drag.key) else {
            return Propagation::Stop;
        };
        let pointer = drag.pressed_in_content + travelled;
        let passed = runtime.with(self.columns.edges, |edges| {
            if place + 1 < edges.column_count() && pointer > edges.middle_of(place + 1) {
                Some(place + 1)
            } else if place > 0 && pointer < edges.middle_of(place - 1) {
                Some(place - 1)
            } else {
                None
            }
        });
        if let Some(passed) = passed {
            runtime.update(self.columns.columns, |columns| columns.swap(place, passed));
        }
        Propagation::Stop
    }

    /// Sorts the rows by the column: in ascending order, or in the other order when they are
    /// sorted by it already.
    fn sort_by(self, runtime: &mut Runtime<Surfaces>, key: &ColumnKey) {
        runtime.update(self.sort, |sort| {
            let order = match sort {
                Some(sorted) if sorted.column == *key => sorted.order.reversed(),
                _ => SortOrder::Ascending,
            };
            *sort = Some(TableSort {
                column: key.clone(),
                order,
            });
        });
    }
}

impl<ColumnKey> Clone for TableHeader<ColumnKey> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<ColumnKey> Copy for TableHeader<ColumnKey> {}

/// The drag of the right edge of a header cell.
struct ColumnResize<ColumnKey> {
    key: ColumnKey,
    /// Where the pointer went down, across the surface.
    pressed_at: f32,
    width_at_press: f32,
}
