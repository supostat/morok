/// A row of entries of one page: as high as its first entry, filled left to right.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Shelf {
    top: u32,
    height: u32,
    used_width: u32,
}

/// Packs rectangles into a square page by shelves: an entry goes on the lowest shelf that is
/// high enough and has room, and opens a new shelf under the last one when none does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShelfPacker {
    side: u32,
    shelves: Vec<Shelf>,
}

impl ShelfPacker {
    pub(crate) fn new(side: u32) -> Self {
        Self {
            side,
            shelves: Vec::new(),
        }
    }

    pub(crate) fn shelf_count(&self) -> usize {
        self.shelves.len()
    }

    pub(crate) fn clear(&mut self) {
        self.shelves.clear();
    }

    /// The top left corner of a free place for an entry of the given size, or none when the
    /// page has no such place left.
    pub(crate) fn place(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        if width > self.side || height > self.side {
            return None;
        }
        let side = self.side;
        let fitting = self
            .shelves
            .iter_mut()
            .filter(|shelf| shelf.height >= height && side - shelf.used_width >= width)
            .min_by_key(|shelf| shelf.height);
        if let Some(shelf) = fitting {
            let place = (shelf.used_width, shelf.top);
            shelf.used_width += width;
            return Some(place);
        }
        let top = self.shelves.last().map_or(0, |shelf| shelf.top + shelf.height);
        if side - top < height {
            return None;
        }
        self.shelves.push(Shelf {
            top,
            height,
            used_width: width,
        });
        Some((0, top))
    }
}

#[cfg(test)]
mod tests {
    use super::ShelfPacker;

    #[test]
    fn entries_of_one_height_fill_a_shelf_left_to_right() {
        let mut packer = ShelfPacker::new(64);

        let places = [packer.place(20, 10), packer.place(20, 10), packer.place(20, 10)];

        assert_eq!(places, [Some((0, 0)), Some((20, 0)), Some((40, 0))]);
        assert_eq!(packer.shelf_count(), 1);
    }

    #[test]
    fn an_entry_that_does_not_fit_the_shelf_opens_the_next_one_under_it() {
        let mut packer = ShelfPacker::new(64);
        packer.place(40, 10);

        assert_eq!(packer.place(40, 10), Some((0, 10)));
        assert_eq!(packer.shelf_count(), 2);
    }

    #[test]
    fn a_lower_entry_goes_on_the_lowest_shelf_that_holds_it() {
        let mut packer = ShelfPacker::new(64);
        packer.place(10, 12);
        packer.place(10, 30);

        assert_eq!(packer.place(10, 10), Some((10, 0)));
        assert_eq!(packer.place(10, 20), Some((10, 12)));
    }

    #[test]
    fn a_higher_entry_opens_a_shelf_of_its_own() {
        let mut packer = ShelfPacker::new(64);
        packer.place(10, 10);

        assert_eq!(packer.place(10, 20), Some((0, 10)));
    }

    #[test]
    fn a_full_page_has_no_place() {
        let mut packer = ShelfPacker::new(32);
        packer.place(32, 20);

        assert_eq!(packer.place(8, 16), None);
        assert_eq!(packer.place(40, 4), None);
    }

    #[test]
    fn a_cleared_page_is_empty_again() {
        let mut packer = ShelfPacker::new(32);
        packer.place(32, 32);

        packer.clear();

        assert_eq!(packer.place(32, 32), Some((0, 0)));
    }
}
