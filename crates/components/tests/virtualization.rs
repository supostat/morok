mod support;

use std::borrow::Cow;
use std::fmt::Write;

use morok_components::{List, RowSelection};
use morok_core::{DisplayItem, ElementId, Offset, Point, ScrollInput, ScrollPhase};
use morok_style::{Declaration, LengthPercentage, LengthPercentageAuto, Pixels};
use support::Stage;

const ROWS: usize = 1_000_000;
const ROW_HEIGHT: f32 = 28.0;
/// The rows a list keeps from the start, and those it keeps past the ones its viewport shows.
const FIRST_POOL: usize = 32;
const OVERSCAN: usize = 4;

fn pixels(length: f32) -> LengthPercentageAuto {
    LengthPercentageAuto::LengthPercentage(LengthPercentage {
        pixels: Pixels(length),
        fraction: None,
        tokens: Cow::Borrowed(&[]),
    })
}

/// A list of a million rows in a view of 200 by `height` pixels.
struct MillionRows {
    stage: Stage,
    view: ElementId,
    viewport: ElementId,
    /// The element that holds the row slots.
    rows: ElementId,
}

impl MillionRows {
    fn new(height: f32) -> Self {
        let mut stage = Stage::new();
        let view = stage.mount(move |mount| {
            let selected = mount.runtime().signal(Vec::new());
            let view = List::new(
                ROWS,
                |index, label| write!(label, "Row {index}").expect("a string takes what is written to it"),
                selected,
            )
            .selection(RowSelection::Multiple)
            .mount(mount);
            mount
                .build(view)
                .inline(Declaration::Width(pixels(200.0)))
                .inline(Declaration::Height(pixels(height)));
            view
        });
        stage.frame();
        let viewport = stage.children(view)[0];
        let rows = stage.children(viewport)[0];
        Self {
            stage,
            view,
            viewport,
            rows,
        }
    }

    fn slots(&self) -> Vec<ElementId> {
        self.stage.children(self.rows)
    }

    /// The texts of the rows the slots show, by the slot.
    fn shown_rows(&self) -> Vec<String> {
        let tree = self.stage.tree();
        self.slots()
            .into_iter()
            .map(|slot| {
                let row = tree.children(slot)[0];
                tree.text(tree.children(row)[0]).to_owned()
            })
            .collect()
    }

    fn element_count(&self) -> usize {
        let tree = self.stage.tree();
        let mut count = 0;
        let mut pending = vec![self.view];
        while let Some(element) = pending.pop() {
            count += 1;
            pending.extend(tree.children(element));
        }
        count
    }

    /// Scrolls the content to `offset` and runs the frames that show it: one that finds the
    /// limits and one whose rows follow the offset.
    fn scroll_to(&mut self, offset: f64) {
        let viewport = self.viewport;
        self.stage
            .surfaces
            .surface_mut(self.stage.surface)
            .tree_mut()
            .scroll_to(viewport, Offset { x: 0.0, y: offset });
        self.stage.frame();
        self.stage.frame();
    }

    /// Turns the wheel over the list by `delta` pixels down and runs the frame that shows it.
    fn wheel(&mut self, delta: f32) {
        self.stage
            .surfaces
            .surface_mut(self.stage.surface)
            .scroll_input(&ScrollInput {
                position: Point { x: 100.0, y: 100.0 },
                delta: Point { x: 0.0, y: delta },
                phase: ScrollPhase::Discrete,
            });
        self.stage.frame();
    }

    /// How far down the display list moves each slot, in the order it paints them: what the
    /// renderer is given. Every slot is laid out in the same box.
    fn painted_slot_shifts(&self) -> Vec<f64> {
        let tree = self.stage.tree();
        let laid_out = tree.border_box(self.slots()[0]);
        tree.display_list()
            .items()
            .iter()
            .filter_map(|item| match item {
                DisplayItem::PushGroup { effects } if effects.frame.border_box == laid_out => {
                    Some(effects.transform.ty)
                }
                _ => None,
            })
            .collect()
    }

    /// The text of every row the viewport shows, with where the top of its slot lies on the
    /// surface.
    fn row_tops_in_view(&self) -> Vec<(String, f32)> {
        let tree = self.stage.tree();
        let viewport = tree.surface_box(self.viewport);
        self.slots()
            .into_iter()
            .filter_map(|slot| {
                let place = tree.surface_box(slot);
                let row = tree.children(slot)[0];
                (place.bottom() > viewport.y && place.y < viewport.bottom())
                    .then(|| (tree.text(tree.children(row)[0]).to_owned(), place.y))
            })
            .collect()
    }

    /// Whether a row stands in a slot the surface shows where its row lies along the content.
    fn shows_row_in_place(&self, index: usize) -> bool {
        let tree = self.stage.tree();
        let label = format!("Row {index}");
        self.slots().into_iter().any(|slot| {
            let row = tree.children(slot)[0];
            tree.text(tree.children(row)[0]) == label
                && (tree.surface_box(slot).y - tree.surface_box(self.rows).y - index as f32 * ROW_HEIGHT).abs() < 0.5
        })
    }
}

#[test]
fn a_million_rows_have_elements_for_the_rows_in_view_alone() {
    let list = MillionRows::new(280.0);

    assert_eq!(list.slots().len(), FIRST_POOL, "ten rows in view keep the first pool");
    assert!(
        list.element_count() < 4 * FIRST_POOL + 16,
        "{} elements",
        list.element_count()
    );
    assert_eq!(
        list.stage.tree().border_box(list.rows).height.to_bits(),
        (ROWS as f32 * ROW_HEIGHT).to_bits(),
        "the content is as long as every row"
    );
    assert!((0..10).all(|index| list.shows_row_in_place(index)));
}

#[test]
fn a_scroll_of_one_row_binds_one_slot_to_the_text_and_the_name_of_the_row_that_came_into_view() {
    let mut list = MillionRows::new(280.0);
    let slots = list.slots();
    let before = list.shown_rows();

    list.stage
        .surfaces
        .surface_mut(list.stage.surface)
        .scroll_input(&ScrollInput {
            position: Point { x: 100.0, y: 100.0 },
            delta: Point { x: 0.0, y: ROW_HEIGHT },
            phase: ScrollPhase::Discrete,
        });
    assert_eq!(
        list.stage.tree().scroll_offset(list.viewport),
        Offset {
            x: 0.0,
            y: f64::from(ROW_HEIGHT)
        },
        "the wheel over the list scrolls it"
    );
    list.stage.frame();

    assert_eq!(list.slots(), slots, "every slot stays");
    let after = list.shown_rows();
    let rebound: Vec<usize> = (0..slots.len()).filter(|slot| before[*slot] != after[*slot]).collect();
    assert_eq!(
        rebound.len(),
        1,
        "one slot shows another row: {before:?} then {after:?}"
    );
    let came_into_view = format!("Row {FIRST_POOL}");
    assert_eq!(after[rebound[0]], came_into_view);
    assert!(!after.contains(&String::from("Row 0")));
    let row = list.stage.tree().children(slots[rebound[0]])[0];
    assert_eq!(
        list.stage.accessible_node(row).label(),
        Some(came_into_view.as_str()),
        "an assistive technology hears the row the slot shows now"
    );
    assert!(list.shows_row_in_place(1) && list.shows_row_in_place(10));
}

#[test]
fn deep_in_the_rows_and_at_their_end_the_same_slots_show_the_rows_in_view() {
    let mut list = MillionRows::new(280.0);
    let slots = list.slots();
    let elements = list.element_count();

    list.scroll_to(500_000.0 * f64::from(ROW_HEIGHT));
    assert_eq!(list.slots(), slots);
    assert_eq!(list.element_count(), elements);
    assert!((500_000..500_010).all(|index| list.shows_row_in_place(index)));

    list.scroll_to(ROWS as f64 * f64::from(ROW_HEIGHT));
    assert_eq!(list.slots(), slots);
    assert!(list.shows_row_in_place(ROWS - 1), "the last row is in view");
    assert_eq!(list.element_count(), elements);
}

#[test]
fn far_down_a_one_pixel_scroll_moves_the_rows_in_view_by_one_pixel() {
    let mut list = MillionRows::new(280.0);
    let odd_far_down = 16_777_217.0;
    list.scroll_to(odd_far_down);
    assert_eq!(
        list.stage.tree().scroll_offset(list.viewport),
        Offset {
            x: 0.0,
            y: odd_far_down
        },
        "2^24 + 1, which single precision rounds to 2^24"
    );

    for _ in 0..3 {
        let before = list.row_tops_in_view();
        let painted_before = list.painted_slot_shifts();
        list.wheel(1.0);
        let after = list.row_tops_in_view();
        let painted_after = list.painted_slot_shifts();

        assert_eq!(painted_after.len(), list.slots().len());
        assert!(
            painted_before
                .iter()
                .zip(&painted_after)
                .all(|(before, after)| (before - after).to_bits() == 1.0_f64.to_bits()),
            "the renderer is given every slot one pixel higher: {painted_before:?} then {painted_after:?}"
        );

        let moved: Vec<(&String, f32)> = after
            .iter()
            .filter_map(|(row, top)| {
                let (_, top_before) = before.iter().find(|(shown, _)| shown == row)?;
                Some((row, top_before - top))
            })
            .collect();
        assert!(moved.len() >= 10, "the rows in view stay in view: {moved:?}");
        assert!(
            moved.iter().all(|(_, by)| by.to_bits() == 1.0_f32.to_bits()),
            "every row moves up by one pixel: {moved:?}"
        );
    }
}

#[test]
fn a_taller_viewport_keeps_as_many_slots_as_it_shows_rows() {
    let list = MillionRows::new(2800.0);
    let mut list = list;
    list.stage.frame();

    assert_eq!(
        list.slots().len(),
        100 + OVERSCAN,
        "a hundred rows in view and the overscan"
    );
    assert!((0..100).all(|index| list.shows_row_in_place(index)));
}
