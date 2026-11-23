use crate::slot_window::SlotWindow;

/// How many columns a table keeps from the start, before its first frame tells how many its
/// viewport shows: the columns of a table of common width are there in its first frame.
const FIRST_POOL: usize = 8;
/// How many columns a table keeps past those its viewport shows, on either side.
const OVERSCAN: usize = 1;

/// Where the columns of a table lie across its content, in the order they are shown: the left
/// edge of every column, and the right edge of the last.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ColumnEdges {
    edges: Vec<f32>,
}

impl ColumnEdges {
    /// The edges of columns of the given widths, side by side from the left edge of the content.
    pub(crate) fn of_widths(widths: impl Iterator<Item = f32>) -> Self {
        let mut edges = vec![0.0];
        let mut right = 0.0;
        for width in widths {
            right += width;
            edges.push(right);
        }
        Self { edges }
    }

    pub(crate) fn column_count(&self) -> usize {
        self.edges.len() - 1
    }

    pub(crate) fn left_of(&self, column: usize) -> f32 {
        self.edges[column]
    }

    pub(crate) fn width_of(&self, column: usize) -> f32 {
        self.edges[column + 1] - self.edges[column]
    }

    /// Where the middle of the column lies.
    pub(crate) fn middle_of(&self, column: usize) -> f32 {
        self.edges[column].midpoint(self.edges[column + 1])
    }

    /// How wide all the columns are together.
    pub(crate) fn total_width(&self) -> f32 {
        self.edges[self.column_count()]
    }

    /// The columns a table has cell slots for while a viewport `viewport_width` wide is
    /// scrolled `offset` across: those in view and the overscan around them.
    pub(crate) fn window(&self, offset: f64, viewport_width: f32) -> SlotWindow {
        let count = self.column_count();
        let rights = &self.edges[1..];
        let lefts = &self.edges[..count];
        let view_end = offset + f64::from(viewport_width);
        let first_in_view = rights.partition_point(|right| f64::from(*right) <= offset);
        let after_last_in_view = lefts.partition_point(|left| f64::from(*left) < view_end);
        let in_view = after_last_in_view.saturating_sub(first_in_view);
        let pool = (in_view + 2 * OVERSCAN).max(FIRST_POOL).min(count);
        SlotWindow {
            first: first_in_view.saturating_sub(OVERSCAN).min(count - pool),
            pool,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ColumnEdges;
    use crate::slot_window::SlotWindow;

    fn hundred_columns() -> ColumnEdges {
        ColumnEdges::of_widths((0..100).map(|_| 50.0))
    }

    #[test]
    fn columns_lie_side_by_side_from_the_left_edge() {
        let edges = ColumnEdges::of_widths([120.0, 80.0, 40.0].into_iter());

        assert_eq!(edges.column_count(), 3);
        assert_eq!(
            (edges.left_of(1), edges.width_of(1), edges.middle_of(1)),
            (120.0, 80.0, 160.0)
        );
        assert_eq!(edges.total_width().to_bits(), 240.0_f32.to_bits());
    }

    #[test]
    fn a_table_of_few_columns_keeps_a_slot_for_each() {
        let edges = ColumnEdges::of_widths([120.0, 80.0, 40.0].into_iter());

        assert_eq!(edges.window(0.0, 0.0), SlotWindow { first: 0, pool: 3 });
        assert_eq!(edges.window(100.0, 60.0), SlotWindow { first: 0, pool: 3 });
    }

    #[test]
    fn a_table_of_many_columns_keeps_slots_for_those_in_view_and_one_past_either_side() {
        let edges = hundred_columns();

        assert_eq!(
            edges.window(0.0, 0.0),
            SlotWindow { first: 0, pool: 8 },
            "before the first frame"
        );
        assert_eq!(edges.window(1000.0, 600.0), SlotWindow { first: 19, pool: 14 });
        assert_eq!(
            edges.window(4400.0, 600.0),
            SlotWindow { first: 86, pool: 14 },
            "at the end"
        );
    }
}
