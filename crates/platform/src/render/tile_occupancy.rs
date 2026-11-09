use crate::render::pixel_rect::PixelRect;

/// The side of a tile in device pixels: fine enough to tell neighbouring boxes of a dense
/// layout apart, coarse enough to keep the map small.
const TILE_SIDE: usize = 8;
const BITS_PER_WORD: usize = 64;

/// Which tiles of a render target a set of rectangles has touched, as one bit per tile.
#[derive(Debug, Default)]
pub(crate) struct TileOccupancy {
    columns: usize,
    rows: usize,
    words_per_row: usize,
    words: Vec<u64>,
    is_empty: bool,
}

/// The words of one row of tiles a span of columns falls into, with the bits of the span
/// masked in the first and the last of them; the words between are covered whole.
#[derive(Clone, Copy, Debug)]
struct ColumnSpan {
    first_word: usize,
    last_word: usize,
    first_mask: u64,
    last_mask: u64,
}

impl TileOccupancy {
    /// Empties the map and sizes it for a target of the given pixels.
    pub(crate) fn reset(&mut self, target: PixelRect) {
        self.columns = target.width.div_ceil(TILE_SIDE);
        self.rows = target.height.div_ceil(TILE_SIDE);
        self.words_per_row = self.columns.div_ceil(BITS_PER_WORD);
        self.words.clear();
        self.words.resize(self.rows * self.words_per_row, 0);
        self.is_empty = true;
    }

    pub(crate) fn clear(&mut self) {
        if !self.is_empty {
            self.words.fill(0);
            self.is_empty = true;
        }
    }

    /// Marks every tile the pixels touch.
    pub(crate) fn mark(&mut self, pixels: PixelRect) {
        let Some(tiles) = self.tiles_of(pixels) else {
            return;
        };
        self.is_empty = false;
        let span = ColumnSpan::of(tiles.x, tiles.x + tiles.width);
        for row in tiles.y..tiles.y + tiles.height {
            let row_start = row * self.words_per_row;
            if span.first_word == span.last_word {
                self.words[row_start + span.first_word] |= span.first_mask & span.last_mask;
                continue;
            }
            self.words[row_start + span.first_word] |= span.first_mask;
            for word in span.first_word + 1..span.last_word {
                self.words[row_start + word] = u64::MAX;
            }
            self.words[row_start + span.last_word] |= span.last_mask;
        }
    }

    /// Whether a marked tile lies under the pixels.
    pub(crate) fn touches(&self, pixels: PixelRect) -> bool {
        if self.is_empty {
            return false;
        }
        let Some(tiles) = self.tiles_of(pixels) else {
            return false;
        };
        let span = ColumnSpan::of(tiles.x, tiles.x + tiles.width);
        for row in tiles.y..tiles.y + tiles.height {
            let row_start = row * self.words_per_row;
            if span.first_word == span.last_word {
                if self.words[row_start + span.first_word] & span.first_mask & span.last_mask != 0 {
                    return true;
                }
                continue;
            }
            let first_touched = self.words[row_start + span.first_word] & span.first_mask != 0;
            let last_touched = self.words[row_start + span.last_word] & span.last_mask != 0;
            let between_touched = self.words[row_start + span.first_word + 1..row_start + span.last_word]
                .iter()
                .any(|word| *word != 0);
            if first_touched || last_touched || between_touched {
                return true;
            }
        }
        false
    }

    /// The tiles the pixels touch, within the map; none when the pixels lie outside it or
    /// have no area.
    fn tiles_of(&self, pixels: PixelRect) -> Option<PixelRect> {
        if pixels.is_empty() {
            return None;
        }
        let first_column = (pixels.x / TILE_SIDE).min(self.columns);
        let first_row = (pixels.y / TILE_SIDE).min(self.rows);
        let end_column = (pixels.x + pixels.width).div_ceil(TILE_SIDE).min(self.columns);
        let end_row = (pixels.y + pixels.height).div_ceil(TILE_SIDE).min(self.rows);
        let tiles = PixelRect {
            x: first_column,
            y: first_row,
            width: end_column.saturating_sub(first_column),
            height: end_row.saturating_sub(first_row),
        };
        (!tiles.is_empty()).then_some(tiles)
    }
}

impl ColumnSpan {
    /// The span of the columns `first..end`; `end` is past `first`.
    fn of(first: usize, end: usize) -> Self {
        let last = end - 1;
        let bits_below_first = first % BITS_PER_WORD;
        let bits_through_last = last % BITS_PER_WORD + 1;
        Self {
            first_word: first / BITS_PER_WORD,
            last_word: last / BITS_PER_WORD,
            first_mask: u64::MAX << bits_below_first,
            last_mask: u64::MAX >> (BITS_PER_WORD - bits_through_last),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TileOccupancy;
    use crate::render::pixel_rect::PixelRect;

    fn pixels(x: usize, y: usize, width: usize, height: usize) -> PixelRect {
        PixelRect { x, y, width, height }
    }

    fn map_of(width: usize, height: usize) -> TileOccupancy {
        let mut occupancy = TileOccupancy::default();
        occupancy.reset(pixels(0, 0, width, height));
        occupancy
    }

    #[test]
    fn an_empty_map_touches_nothing() {
        let occupancy = map_of(1000, 1000);

        assert!(!occupancy.touches(pixels(0, 0, 1000, 1000)));
    }

    #[test]
    fn marked_pixels_are_touched_by_a_rectangle_over_them_and_not_by_one_beside_them() {
        let mut occupancy = map_of(1000, 1000);
        occupancy.mark(pixels(100, 100, 10, 10));

        assert!(occupancy.touches(pixels(105, 105, 1, 1)));
        assert!(occupancy.touches(pixels(0, 0, 101, 101)));
        assert!(!occupancy.touches(pixels(120, 100, 10, 10)));
        assert!(!occupancy.touches(pixels(100, 120, 10, 10)));
    }

    #[test]
    fn a_rectangle_in_the_same_tile_counts_as_touching() {
        let mut occupancy = map_of(1000, 1000);
        occupancy.mark(pixels(100, 100, 2, 2));

        assert!(occupancy.touches(pixels(102, 102, 2, 2)));
    }

    #[test]
    fn marks_span_the_words_of_a_wide_row() {
        let mut occupancy = map_of(2000, 100);
        occupancy.mark(pixels(500, 10, 800, 10));

        assert!(occupancy.touches(pixels(1200, 15, 1, 1)));
        assert!(occupancy.touches(pixels(504, 15, 1, 1)));
        assert!(occupancy.touches(pixels(1296, 15, 1, 1)));
        assert!(!occupancy.touches(pixels(1400, 15, 1, 1)));
        assert!(!occupancy.touches(pixels(400, 15, 1, 1)));
    }

    #[test]
    fn a_wide_rectangle_touches_a_mark_between_its_first_and_last_word() {
        let mut occupancy = map_of(2000, 100);
        occupancy.mark(pixels(1000, 10, 8, 8));

        assert!(occupancy.touches(pixels(100, 0, 1800, 50)));
        assert!(!occupancy.touches(pixels(100, 50, 1800, 50)));
    }

    #[test]
    fn pixels_beyond_the_target_are_ignored() {
        let mut occupancy = map_of(100, 100);
        occupancy.mark(pixels(90, 90, 100, 100));

        assert!(occupancy.touches(pixels(95, 95, 1, 1)));
        assert!(!occupancy.touches(pixels(200, 200, 10, 10)));
    }

    #[test]
    fn clearing_forgets_every_mark() {
        let mut occupancy = map_of(100, 100);
        occupancy.mark(pixels(10, 10, 10, 10));
        occupancy.clear();

        assert!(!occupancy.touches(pixels(10, 10, 10, 10)));
    }
}
