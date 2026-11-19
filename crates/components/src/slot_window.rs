/// The items a virtualized view has slots for: as many as the pool holds, from `first` on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SlotWindow {
    pub(crate) first: usize,
    pub(crate) pool: usize,
}

impl SlotWindow {
    /// The item a slot shows: each the item of the window that leaves the number of the slot
    /// when divided by the size of the pool, so that a scroll rebinds only the slots whose
    /// items left the window.
    pub(crate) fn item_of_slot(self, slot: usize) -> usize {
        // A slot about to go with a pool that shrank to nothing reads the window once more.
        self.first.checked_rem(self.pool).map_or(self.first, |first_slot| {
            self.first + (slot + self.pool - first_slot) % self.pool
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SlotWindow;

    #[test]
    fn a_window_that_moves_by_one_item_rebinds_one_slot() {
        let before = SlotWindow { first: 6, pool: 4 };
        let after = SlotWindow { first: 7, pool: 4 };

        let items = |window: SlotWindow| (0..4).map(|slot| window.item_of_slot(slot)).collect::<Vec<_>>();

        assert_eq!(items(before), [8, 9, 6, 7]);
        assert_eq!(items(after), [8, 9, 10, 7]);
    }
}
