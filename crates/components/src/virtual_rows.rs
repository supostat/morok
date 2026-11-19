use morok_core::{ElementId, Point};
use morok_reactive::{Memo, Signal, Source};
use morok_style::{Declaration, Position, Side};

use crate::length_in_pixels::pixels;
use crate::scroll_frame::ScrollFrame;
use crate::slot_window::SlotWindow;
use crate::view::{Mount, ScrollArea};

/// How many rows a view keeps from the start, before its first frame tells how many its
/// viewport shows: the rows of a view of common height are there in its first frame.
const FIRST_POOL: usize = 32;
/// How many rows a view keeps past those its viewport shows, so that a scroll between two
/// frames does not show a gap.
const OVERSCAN: usize = 4;

/// Rows of one height in a view whose content scrolls, of which only those the viewport shows
/// have elements: a pool of row slots, each moved to the row it shows. A scroll binds the slots
/// that leave the viewport to the rows that come into it; the other slots keep their rows, and
/// no slot is built again. The number of elements depends on the viewport, never on the rows.
///
/// Every slot is laid out at the top of the rows and moved to its row as a picture, so a scroll
/// lays out no box, and what a slot holds keeps the small coordinates layout gave it however
/// far down its row lies.
#[derive(Clone, Copy, Debug)]
pub(crate) struct VirtualRows {
    pub(crate) frame: ScrollFrame,
    /// How far the viewport is scrolled and how large it is, as the last frame found them.
    pub(crate) area: Signal<ScrollArea>,
    row_height: f32,
    /// How much of the top of the viewport something that stays there covers: a header.
    covered_top: f32,
}

impl VirtualRows {
    /// Builds the view of `count` rows `row_height` high, the rows being all its content:
    /// `row` builds the content of a slot, given the index of the row the slot shows.
    pub(crate) fn mount(
        mount: &mut Mount<'_>,
        count: Source<usize>,
        row_height: f32,
        row: impl Fn(&mut Mount<'_>, Memo<usize>) -> ElementId + 'static,
    ) -> Self {
        let rows = mount.element().id();
        let frame = ScrollFrame::mount(mount, rows);
        let view = Self::of_frame(mount, frame, row_height, 0.0);
        view.fill(mount, rows, count, row);
        view
    }

    /// The rows of the view of `frame`, each `row_height` high. They begin `covered_top` below
    /// the top of the content, under something that stays at the top of the viewport.
    pub(crate) fn of_frame(mount: &mut Mount<'_>, frame: ScrollFrame, row_height: f32, covered_top: f32) -> Self {
        Self {
            frame,
            area: mount.follow_scroll(frame.viewport),
            row_height,
            covered_top,
        }
    }

    /// Fills `rows`, an element of the content of the view, with the slots of `count` rows and
    /// makes it as high as all of them: `row` builds the content of a slot, given the index of
    /// the row the slot shows.
    pub(crate) fn fill(
        self,
        mount: &mut Mount<'_>,
        rows: ElementId,
        count: Source<usize>,
        row: impl Fn(&mut Mount<'_>, Memo<usize>) -> ElementId + 'static,
    ) {
        let Self { area, row_height, .. } = self;
        let runtime = mount.runtime();
        let window = runtime.memo(move |runtime| {
            let area = runtime.get(area);
            let count = runtime.get(count);
            let pool = (rows_in(area.viewport.height, row_height) + OVERSCAN)
                .max(FIRST_POOL)
                .min(count);
            let first = whole_rows_in(area.offset.y.max(0.0), row_height).min(count - pool);
            SlotWindow { first, pool }
        });
        let pool = runtime.memo(move |runtime| runtime.get(window).pool);
        let slots = runtime.memo(move |runtime| (0..runtime.get(pool)).collect::<Vec<usize>>());
        let height =
            runtime.memo(move |runtime| Declaration::Height(pixels(length_of(runtime.get(count), row_height))));
        mount
            .build(rows)
            .inline(Declaration::Position(Position::Relative))
            .inline(height)
            .for_each(
                slots,
                |slot| *slot,
                move |mount, slot| {
                    let slot = *slot;
                    let index = mount
                        .runtime()
                        .memo(move |runtime| runtime.get(window).item_of_slot(slot));
                    let content = row(mount, index);
                    let slot_element = mount
                        .element()
                        .inline(Declaration::Position(Position::Absolute))
                        .inline(Declaration::Inset(Side::Top, pixels(0.0)))
                        .inline(Declaration::Inset(Side::Left, pixels(0.0)))
                        .inline(Declaration::Inset(Side::Right, pixels(0.0)))
                        .inline(Declaration::Height(pixels(row_height)))
                        .child(content)
                        .id();
                    mount.effect(move |runtime, tree| {
                        let row_top = length_of(runtime.get(index), row_height);
                        tree.set_displacement(slot_element, Point { x: 0.0, y: row_top });
                    });
                    slot_element
                },
            );
    }

    /// Scrolls the view as little as brings the row into view whenever `row` changes: the row
    /// the keyboard is on.
    pub(crate) fn keep_in_view(self, mount: &mut Mount<'_>, row: Signal<usize>) {
        let Self {
            frame,
            area,
            row_height,
            covered_top,
        } = self;
        mount.effect(move |runtime, tree| {
            let top = length_of(runtime.get(row), row_height);
            let area = runtime.untracked(|runtime| runtime.get(area));
            let offset = area.offset.y;
            let height = area.viewport.height - covered_top;
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
