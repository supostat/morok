use morok_core::{Key, KeyEvent, KeyPhase, Modifiers, Propagation};
use morok_reactive::{Memo, Runtime, Signal};

use crate::row_selection::RowSelection;
use crate::view::Surfaces;

/// What the user does to the selection of the rows of a list or a table.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RowChoice {
    /// The selected rows, in ascending order.
    pub(crate) selected: Signal<Vec<usize>>,
    pub(crate) selection: RowSelection,
    /// The row the keyboard is on.
    pub(crate) keyboard_row: Signal<usize>,
    /// Where a selection by Shift begins.
    anchor: Signal<usize>,
}

impl RowChoice {
    pub(crate) fn new(runtime: &mut Runtime<Surfaces>, selected: Signal<Vec<usize>>, selection: RowSelection) -> Self {
        Self {
            selected,
            selection,
            keyboard_row: runtime.signal(0),
            anchor: runtime.signal(0),
        }
    }

    /// Whether the row a slot shows is selected.
    pub(crate) fn is_selected(self, runtime: &mut Runtime<Surfaces>, index: Memo<usize>) -> Memo<bool> {
        let selected = self.selected;
        runtime.memo(move |runtime| {
            let index = runtime.get(index);
            runtime.with(selected, |selected| selected.binary_search(&index).is_ok())
        })
    }

    pub(crate) fn click(self, runtime: &mut Runtime<Surfaces>, index: usize, modifiers: Modifiers) {
        let extends = self.selection == RowSelection::Multiple;
        if extends && modifiers.contains(Modifiers::SHIFT) {
            self.select_from_anchor(runtime, index);
        } else if extends && modifiers.contains(Modifiers::COMMAND) {
            self.toggle(runtime, index);
            runtime.set(self.anchor, index);
        } else {
            self.select_only(runtime, index);
        }
        runtime.set(self.keyboard_row, index);
    }

    /// What a key going down does to the selection of `count` rows.
    pub(crate) fn key(self, runtime: &mut Runtime<Surfaces>, key: &KeyEvent, count: usize) -> Propagation {
        if key.phase != KeyPhase::Pressed || count == 0 {
            return Propagation::Continue;
        }
        let now = runtime.get(self.keyboard_row).min(count - 1);
        let extends = self.selection == RowSelection::Multiple && key.modifiers == Modifiers::SHIFT;
        let moved_to = match (key.key, key.modifiers) {
            (Key::ArrowDown, Modifiers::NONE | Modifiers::SHIFT) => (now + 1).min(count - 1),
            (Key::ArrowUp, Modifiers::NONE | Modifiers::SHIFT) => now.saturating_sub(1),
            (Key::Home, Modifiers::NONE | Modifiers::SHIFT) => 0,
            (Key::End, Modifiers::NONE | Modifiers::SHIFT) => count - 1,
            (Key::Space, Modifiers::NONE) => {
                match self.selection {
                    RowSelection::Single => self.select_only(runtime, now),
                    RowSelection::Multiple => self.toggle(runtime, now),
                }
                return Propagation::Stop;
            }
            _ => return Propagation::Continue,
        };
        if extends {
            self.select_from_anchor(runtime, moved_to);
        } else {
            self.select_only(runtime, moved_to);
        }
        runtime.set(self.keyboard_row, moved_to);
        Propagation::Stop
    }

    fn select_only(self, runtime: &mut Runtime<Surfaces>, index: usize) {
        runtime.set(self.selected, vec![index]);
        runtime.set(self.anchor, index);
    }

    fn select_from_anchor(self, runtime: &mut Runtime<Surfaces>, index: usize) {
        let anchor = runtime.get(self.anchor);
        runtime.set(self.selected, (anchor.min(index)..=anchor.max(index)).collect());
    }

    fn toggle(self, runtime: &mut Runtime<Surfaces>, index: usize) {
        runtime.update(self.selected, |selected| match selected.binary_search(&index) {
            Ok(position) => {
                selected.remove(position);
            }
            Err(position) => selected.insert(position, index),
        });
    }
}
