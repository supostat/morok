use morok_core::{ElementId, Point};
use morok_reactive::{Memo, Signal, Source};
use morok_style::{Declaration, Position, Side};

use crate::length_in_pixels::pixels;
use crate::scroll_frame::ScrollFrame;
use crate::view::{Mount, ScrollArea};

/// How many rows a view keeps from the start, before its first frame tells how many its
/// viewport shows: the rows of a view of common height are there in its first frame.
const FIRST_POOL: usize = 32;
/// How many rows a view keeps past those its viewport shows, so that a scroll between two
/// frames does not show a gap.
const OVERSCAN: usize = 4;

/// Rows of one height in a view whose content scrolls, of which only those the viewport shows
/// have elements: a pool of row slots, each placed at the row it shows. A scroll binds the slots
/// that leave the viewport to the rows that come into it; the other slots keep their rows, and
/// no slot is built again. The number of elements depends on the viewport, never on the rows.
#[derive(Clone, Copy, Debug)]
pub(crate) struct VirtualRows {
    pub(crate) frame: ScrollFrame,
    /// How far the viewport is scrolled and how large it is, as the last frame found them.
    pub(crate) area: Signal<ScrollArea>,
    row_height: f32,
}

impl VirtualRows {
    /// Builds the view of `count` rows `row_height` high: `row` builds the content of a slot,
    /// given the index of the row the slot shows.
    pub(crate) fn mount(
        mount: &mut Mount<'_>,
        count: Source<usize>,
        row_height: f32,
        row: impl Fn(&mut Mount<'_>, Memo<usize>) -> ElementId + 'static,
    ) -> Self {
        let rows = mount.element().inline(Declaration::Position(Position::Relative)).id();
        let frame = ScrollFrame::mount(mount, rows);
        let area = mount.follow_scroll(frame.viewport);
        let runtime = mount.runtime();
        let window = runtime.memo(move |runtime| {
            let area = runtime.get(area);
            let count = runtime.get(count);
            let pool = (rows_in(area.viewport.height, row_height) + OVERSCAN)
                .max(FIRST_POOL)
                .min(count);
            let first = whole_rows_in(area.offset.y.max(0.0), row_height).min(count - pool);
            RowWindow { first, pool }
        });
        let slots = runtime.memo(move |runtime| (0..runtime.get(window).pool).collect::<Vec<usize>>());
        let height =
            runtime.memo(move |runtime| Declaration::Height(pixels(length_of(runtime.get(count), row_height))));
        mount.build(rows).inline(height).for_each(
            slots,
            |slot| *slot,
            move |mount, slot| {
                let slot = *slot;
                let runtime = mount.runtime();
                let index = runtime.memo(move |runtime| runtime.get(window).row_of_slot(slot));
                let place = runtime.memo(move |runtime| {
                    Declaration::Inset(Side::Top, pixels(length_of(runtime.get(index), row_height)))
                });
                let content = row(mount, index);
                mount
                    .element()
                    .inline(Declaration::Position(Position::Absolute))
                    .inline(Declaration::Inset(Side::Left, pixels(0.0)))
                    .inline(Declaration::Inset(Side::Right, pixels(0.0)))
                    .inline(Declaration::Height(pixels(row_height)))
                    .inline(place)
                    .child(content)
                    .id()
            },
        );
        Self {
            frame,
            area,
            row_height,
        }
    }

    /// Scrolls the view as little as brings the row into view whenever `row` changes: the row
    /// the keyboard is on.
    pub(crate) fn keep_in_view(self, mount: &mut Mount<'_>, row: Signal<usize>) {
        let Self {
            frame,
            area,
            row_height,
        } = self;
        mount.effect(move |runtime, tree| {
            let top = length_of(runtime.get(row), row_height);
            let area = runtime.untracked(|runtime| runtime.get(area));
            let (offset, height) = (area.offset.y, area.viewport.height);
            let goal = if height <= 0.0 {
                None
            } else if top < offset {
                Some(top)
            } else if top + row_height > offset + height {
                Some(top + row_height - height)
            } else {
                None
            };
            if let Some(goal) = goal {
                tree.scroll_to(
                    frame.viewport,
                    Point {
                        x: area.offset.x,
                        y: goal,
                    },
                );
            }
        });
    }
}

/// The rows a view has slots for: as many as the pool holds, from `first` on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RowWindow {
    first: usize,
    pool: usize,
}

impl RowWindow {
    /// The row a slot shows: each the row of the window that leaves the number of the slot
    /// when divided by the size of the pool, so that a scroll rebinds only the slots whose
    /// rows left the window.
    fn row_of_slot(self, slot: usize) -> usize {
        // A slot about to go with a pool that shrank to nothing reads the window once more.
        self.first.checked_rem(self.pool).map_or(self.first, |first_slot| {
            self.first + (slot + self.pool - first_slot) % self.pool
        })
    }
}

/// How many whole or partial rows of `row_height` a length holds.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a length on a surface is far below the largest count of rows and not negative"
)]
fn rows_in(length: f32, row_height: f32) -> usize {
    (length / row_height).ceil() as usize
}

/// How many whole rows of `row_height` a length holds.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a length on a surface is far below the largest count of rows and not negative"
)]
fn whole_rows_in(length: f32, row_height: f32) -> usize {
    (length / row_height).floor() as usize
}

/// How long `rows` rows of `row_height` are.
#[expect(clippy::cast_precision_loss, reason = "lengths on a surface are single precision")]
fn length_of(rows: usize, row_height: f32) -> f32 {
    rows as f32 * row_height
}
