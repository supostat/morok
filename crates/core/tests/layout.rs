mod support;

use std::borrow::Cow;

use morok_core::{Rect, Role, Size};
use morok_style::{Declaration, LengthPercentage, Pixels, RepeatCount, States, TrackBreadth, TrackListItem, TrackSize};
use support::{CHARACTER_WIDTH, LINE_HEIGHT, Scene};

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(x, y, width, height)
}

#[test]
fn blocks_stack_down_the_page_and_fill_the_width_of_their_container() {
    let mut scene = Scene::new(
        ".page { padding: var(--space-4); }\n\
         .row { height: 30px; }\n\
         .spaced { margin: 10px 0 20px; height: 40px; }",
    );
    let page = scene.add(scene.root(), Role::Element, &["page"]);
    let first = scene.add(page, Role::Element, &["row"]);
    let spaced = scene.add(page, Role::Element, &["spaced"]);
    let last = scene.add(page, Role::Element, &["row"]);

    scene.frame();

    assert_eq!(scene.border_box(page), rect(0.0, 0.0, 800.0, 162.0));
    assert_eq!(scene.border_box(first), rect(16.0, 16.0, 768.0, 30.0));
    assert_eq!(scene.border_box(spaced), rect(16.0, 56.0, 768.0, 40.0));
    assert_eq!(scene.border_box(last), rect(16.0, 116.0, 768.0, 30.0));
}

#[test]
fn the_margins_of_adjoining_blocks_collapse() {
    let mut scene = Scene::new(
        ".above { height: 10px; margin-bottom: 20px; }\n\
         .below { height: 10px; margin-top: 30px; }\n\
         .pushed { height: 10px; margin-top: 12px; }",
    );
    let above = scene.add(scene.root(), Role::Element, &["above"]);
    let below = scene.add(scene.root(), Role::Element, &["below"]);
    let wrapper = scene.add(scene.root(), Role::Element, &[]);
    let pushed = scene.add(wrapper, Role::Element, &["pushed"]);

    scene.frame();

    assert_eq!(scene.border_box(above), rect(0.0, 0.0, 800.0, 10.0));
    assert_eq!(scene.border_box(below), rect(0.0, 40.0, 800.0, 10.0));
    assert_eq!(scene.border_box(wrapper), rect(0.0, 62.0, 800.0, 10.0));
    assert_eq!(scene.border_box(pushed), rect(0.0, 62.0, 800.0, 10.0));
}

#[test]
fn a_flex_row_places_its_items_with_gaps_growth_and_alignment() {
    let mut scene = Scene::new(
        ".toolbar { display: flex; align-items: center; gap: var(--space-2); height: 48px; \
         padding: 0 var(--space-3); }\n\
         .button { width: 80px; height: 28px; }\n\
         .filler { flex: 1; height: 28px; }",
    );
    let toolbar = scene.add(scene.root(), Role::Element, &["toolbar"]);
    let first = scene.add(toolbar, Role::Element, &["button"]);
    let filler = scene.add(toolbar, Role::Element, &["filler"]);
    let last = scene.add(toolbar, Role::Element, &["button"]);

    scene.frame();

    assert_eq!(scene.border_box(toolbar), rect(0.0, 0.0, 800.0, 48.0));
    assert_eq!(scene.border_box(first), rect(12.0, 10.0, 80.0, 28.0));
    assert_eq!(scene.border_box(filler), rect(100.0, 10.0, 600.0, 28.0));
    assert_eq!(scene.border_box(last), rect(708.0, 10.0, 80.0, 28.0));
}

#[test]
fn a_flex_column_distributes_free_space_and_aligns_across() {
    let mut scene = Scene::new(
        ".column { display: flex; flex-direction: column; justify-content: space-between; \
         align-items: flex-end; width: 100px; height: 200px; }\n\
         .item { width: 40px; height: 20px; }",
    );
    let column = scene.add(scene.root(), Role::Element, &["column"]);
    let items = [(); 3].map(|()| scene.add(column, Role::Element, &["item"]));

    scene.frame();

    assert_eq!(scene.border_box(items[0]), rect(60.0, 0.0, 40.0, 20.0));
    assert_eq!(scene.border_box(items[1]), rect(60.0, 90.0, 40.0, 20.0));
    assert_eq!(scene.border_box(items[2]), rect(60.0, 180.0, 40.0, 20.0));
}

#[test]
fn a_wrapping_flex_container_breaks_its_items_into_lines() {
    let mut scene = Scene::new(
        ".tiles { display: flex; flex-wrap: wrap; gap: 10px; width: 100px; }\n\
         .tile { width: 45px; height: 20px; }",
    );
    let tiles = scene.add(scene.root(), Role::Element, &["tiles"]);
    let tile = [(); 3].map(|()| scene.add(tiles, Role::Element, &["tile"]));

    scene.frame();

    assert_eq!(scene.border_box(tiles), rect(0.0, 0.0, 100.0, 50.0));
    assert_eq!(scene.border_box(tile[0]), rect(0.0, 0.0, 45.0, 20.0));
    assert_eq!(scene.border_box(tile[1]), rect(55.0, 0.0, 45.0, 20.0));
    assert_eq!(scene.border_box(tile[2]), rect(0.0, 30.0, 45.0, 20.0));
}

#[test]
fn a_flex_item_that_hides_its_overflow_may_shrink_below_its_content() {
    let mut scene = Scene::new(
        ".row { display: flex; width: 100px; }\n\
         .clipped { overflow: hidden; }",
    );
    let row = scene.add(scene.root(), Role::Element, &["row"]);
    let visible = scene.add(row, Role::Element, &[]);
    scene.add_text(visible, "abcdefghijklmnopqrstuvwxyz");
    scene.frame();
    assert_eq!(scene.border_box(visible), rect(0.0, 0.0, 260.0, 20.0));

    scene.add_classes(visible, &["clipped"]);
    scene.frame();

    assert_eq!(scene.border_box(visible), rect(0.0, 0.0, 100.0, 20.0));
}

#[test]
fn a_grid_sizes_its_tracks_and_places_its_items() {
    let mut scene = Scene::new(
        ".grid { display: grid; grid-template-columns: 100px 1fr 2fr; grid-template-rows: 40px 60px; \
         grid-auto-rows: 25px; gap: 10px; width: 420px; }\n\
         .wide { grid-column: 2 / span 2; grid-row: 2; }",
    );
    let grid = scene.add(scene.root(), Role::Element, &["grid"]);
    let cells = [(); 3].map(|()| scene.add(grid, Role::Element, &[]));
    let wide = scene.add(grid, Role::Element, &["wide"]);
    let after = scene.add(grid, Role::Element, &[]);
    let implicit = scene.add(grid, Role::Element, &[]);

    scene.frame();

    assert_eq!(scene.border_box(grid), rect(0.0, 0.0, 420.0, 145.0));
    assert_eq!(scene.border_box(cells[0]), rect(0.0, 0.0, 100.0, 40.0));
    assert_eq!(scene.border_box(cells[1]), rect(110.0, 0.0, 100.0, 40.0));
    assert_eq!(scene.border_box(cells[2]), rect(220.0, 0.0, 200.0, 40.0));
    assert_eq!(scene.border_box(wide), rect(110.0, 50.0, 310.0, 60.0));
    assert_eq!(scene.border_box(after), rect(0.0, 50.0, 100.0, 60.0));
    assert_eq!(scene.border_box(implicit), rect(0.0, 120.0, 100.0, 25.0));
}

#[test]
fn repeated_and_bounded_tracks_share_the_grid() {
    let mut scene = Scene::new(
        ".thirds { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); width: 300px; height: 30px; }\n\
         .sidebar { display: grid; grid-template-columns: fit-content(120px) minmax(50px, 80px) auto; \
         width: 400px; }",
    );
    let thirds = scene.add(scene.root(), Role::Element, &["thirds"]);
    let third = [(); 3].map(|()| scene.add(thirds, Role::Element, &[]));
    let sidebar = scene.add(scene.root(), Role::Element, &["sidebar"]);
    let fitted = scene.add(sidebar, Role::Element, &[]);
    scene.add_text(fitted, "a label that is long");
    let bounded = scene.add(sidebar, Role::Element, &[]);
    let rest = scene.add(sidebar, Role::Element, &[]);

    scene.frame();

    assert_eq!(scene.border_box(third[0]), rect(0.0, 0.0, 100.0, 30.0));
    assert_eq!(scene.border_box(third[1]), rect(100.0, 0.0, 100.0, 30.0));
    assert_eq!(scene.border_box(third[2]), rect(200.0, 0.0, 100.0, 30.0));
    assert_eq!(scene.border_box(fitted), rect(0.0, 30.0, 120.0, 40.0));
    assert_eq!(scene.border_box(bounded), rect(120.0, 30.0, 80.0, 40.0));
    assert_eq!(scene.border_box(rest), rect(200.0, 30.0, 200.0, 40.0));
}

#[test]
fn an_absolute_element_is_placed_against_the_padding_box_of_its_containing_block() {
    let mut scene = Scene::new(
        ".panel { position: relative; width: 300px; height: 200px; margin: 20px; padding: 10px; \
         border: 2px solid var(--color-line); }\n\
         .badge { position: absolute; top: 8px; right: 8px; width: 20px; height: 20px; }\n\
         .cover { position: absolute; inset: 0; }\n\
         .holder { height: 50px; margin-top: 30px; }\n\
         .corner { position: absolute; left: 0; bottom: 0; width: 10px; height: 10px; }",
    );
    let panel = scene.add(scene.root(), Role::Element, &["panel"]);
    let badge = scene.add(panel, Role::Element, &["badge"]);
    let cover = scene.add(panel, Role::Element, &["cover"]);
    let holder = scene.add(panel, Role::Element, &["holder"]);
    let corner = scene.add(holder, Role::Element, &["corner"]);

    scene.frame();

    assert_eq!(scene.border_box(panel), rect(20.0, 20.0, 300.0, 200.0));
    assert_eq!(scene.border_box(badge), rect(290.0, 30.0, 20.0, 20.0));
    assert_eq!(scene.border_box(cover), rect(22.0, 22.0, 296.0, 196.0));
    assert_eq!(scene.border_box(holder), rect(32.0, 62.0, 276.0, 50.0));
    assert_eq!(scene.border_box(corner), rect(22.0, 208.0, 10.0, 10.0));
}

#[test]
fn an_absolute_element_without_a_positioned_ancestor_is_placed_against_the_viewport() {
    let mut scene = Scene::new(
        ".box { height: 100px; margin: 40px; }\n\
         .toast { position: absolute; right: 0; bottom: 0; width: 120px; height: 30px; }\n\
         .next { height: 10px; }",
    );
    let container = scene.add(scene.root(), Role::Element, &["box"]);
    let toast = scene.add(container, Role::Element, &["toast"]);
    let next = scene.add(container, Role::Element, &["next"]);

    scene.frame();

    assert_eq!(scene.border_box(toast), rect(680.0, 570.0, 120.0, 30.0));
    assert_eq!(scene.border_box(next), rect(40.0, 40.0, 720.0, 10.0));
}

#[test]
fn an_absolute_child_of_a_grid_is_placed_against_the_grid_area_it_names() {
    let mut scene = Scene::new(
        ".board { position: relative; display: grid; grid-template-columns: 100px 100px; \
         grid-template-rows: 50px 50px; }\n\
         .pin { position: absolute; grid-column: 2 / 3; grid-row: 2 / 3; inset: 0; }\n\
         .tail { position: absolute; grid-column: 2; grid-row: 2; inset: 0; }",
    );
    let board = scene.add(scene.root(), Role::Element, &["board"]);
    let pin = scene.add(board, Role::Element, &["pin"]);
    let tail = scene.add(board, Role::Element, &["tail"]);

    scene.frame();

    assert_eq!(scene.border_box(board), rect(0.0, 0.0, 800.0, 100.0));
    assert_eq!(scene.border_box(pin), rect(100.0, 50.0, 100.0, 50.0));
    assert_eq!(
        scene.border_box(tail),
        rect(100.0, 50.0, 700.0, 50.0),
        "a line left `auto` is the padding edge of the grid"
    );
}

#[test]
fn calc_lengths_resolve_against_their_percentage_basis() {
    let mut scene = Scene::new(
        ".frame { width: 400px; }\n\
         .inner { width: calc(100% - var(--space-4) * 2); margin-left: calc(10% + 4px); \
         padding-left: calc(5% + 1px); height: 10px; }\n\
         .content { height: 10px; }\n\
         .shelf { display: flex; width: 400px; height: 20px; }\n\
         .based { flex: 0 0 calc(25% + 10px); }",
    );
    let frame = scene.add(scene.root(), Role::Element, &["frame"]);
    let inner = scene.add(frame, Role::Element, &["inner"]);
    let content = scene.add(inner, Role::Element, &["content"]);
    let shelf = scene.add(scene.root(), Role::Element, &["shelf"]);
    let based = scene.add(shelf, Role::Element, &["based"]);

    scene.frame();

    assert_eq!(scene.border_box(inner), rect(44.0, 0.0, 368.0, 10.0));
    assert_eq!(scene.border_box(content), rect(65.0, 0.0, 347.0, 10.0));
    assert_eq!(scene.border_box(based), rect(0.0, 10.0, 110.0, 20.0));
}

#[test]
fn inline_grid_tracks_that_mix_pixels_with_a_percentage_resolve_against_the_grid() {
    let mut scene = Scene::new(".split { display: grid; width: 300px; height: 20px; }");
    let split = scene.add(scene.root(), Role::Element, &["split"]);
    let cells = [(); 4].map(|()| scene.add(split, Role::Element, &[]));
    let mixed = |pixels: f32, fraction: f32| LengthPercentage {
        pixels: Pixels(pixels),
        fraction: Some(fraction),
        tokens: Cow::Borrowed(&[]),
    };

    scene.tree.set_inline(
        split,
        Declaration::GridTemplateColumns(Cow::Owned(vec![
            TrackListItem::Track(TrackSize::Breadth(TrackBreadth::LengthPercentage(mixed(-10.0, 0.5)))),
            TrackListItem::Repeat {
                count: RepeatCount::Times(1),
                tracks: Cow::Owned(vec![
                    TrackSize::MinMax {
                        min: TrackBreadth::LengthPercentage(mixed(5.0, 0.1)),
                        max: TrackBreadth::LengthPercentage(mixed(10.0, 0.1)),
                    },
                    TrackSize::FitContent(mixed(20.0, 0.1)),
                ]),
            },
            TrackListItem::Track(TrackSize::Breadth(TrackBreadth::Flex(1.0))),
        ])),
    );
    scene.frame();

    assert_eq!(scene.border_box(cells[0]), rect(0.0, 0.0, 140.0, 20.0));
    assert_eq!(scene.border_box(cells[1]), rect(140.0, 0.0, 40.0, 20.0));
    assert_eq!(scene.border_box(cells[2]), rect(180.0, 0.0, 0.0, 20.0));
    assert_eq!(scene.border_box(cells[3]), rect(180.0, 0.0, 120.0, 20.0));

    scene.tree.set_inline(
        split,
        Declaration::GridTemplateColumns(Cow::Owned(vec![
            TrackListItem::Track(TrackSize::Breadth(TrackBreadth::LengthPercentage(mixed(10.0, 0.25)))),
            TrackListItem::Track(TrackSize::Breadth(TrackBreadth::Flex(1.0))),
        ])),
    );
    scene.frame();

    assert_eq!(scene.border_box(cells[0]), rect(0.0, 0.0, 85.0, 10.0));
    assert_eq!(scene.border_box(cells[1]), rect(85.0, 0.0, 215.0, 10.0));
    assert_eq!(scene.border_box(cells[2]), rect(0.0, 10.0, 85.0, 10.0));
}

#[test]
fn sizes_are_clamped_by_their_minimum_and_maximum() {
    let mut scene = Scene::new(".clamped { width: 50%; max-width: 300px; min-height: 40px; }");
    let clamped = scene.add(scene.root(), Role::Element, &["clamped"]);

    scene.frame();

    assert_eq!(scene.border_box(clamped), rect(0.0, 0.0, 300.0, 40.0));
}

#[test]
fn the_root_is_the_viewport_whatever_it_lays_out() {
    let mut scene = Scene::new(
        ".bar { display: flex; }\n\
         .half { width: 50%; height: 50%; }",
    );
    let half = scene.add(scene.root(), Role::Element, &["half"]);
    scene.frame();
    assert_eq!(scene.border_box(scene.root()), rect(0.0, 0.0, 800.0, 600.0));
    assert_eq!(scene.border_box(half), rect(0.0, 0.0, 400.0, 300.0));

    scene.add_classes(scene.root(), &["bar"]);
    scene.viewport = Size {
        width: 400.0,
        height: 200.0,
    };
    scene.frame();

    assert_eq!(scene.border_box(scene.root()), rect(0.0, 0.0, 400.0, 200.0));
    assert_eq!(scene.border_box(half), rect(0.0, 0.0, 200.0, 100.0));
}

#[test]
fn a_root_with_a_size_of_its_own_keeps_it() {
    let mut scene = Scene::new(".sheet { width: 320px; height: 50%; }");
    scene.add_classes(scene.root(), &["sheet"]);

    scene.frame();

    assert_eq!(scene.border_box(scene.root()), rect(0.0, 0.0, 320.0, 300.0));
}

#[test]
fn a_text_is_as_large_as_the_measurer_says() {
    let mut scene = Scene::new(
        ".line { display: flex; }\n\
         .narrow { width: 60px; }",
    );
    let line = scene.add(scene.root(), Role::Element, &["line"]);
    let on_one_line = scene.add_text(line, "hello world");
    let narrow = scene.add(scene.root(), Role::Element, &["narrow"]);
    let wrapped = scene.add_text(narrow, "hello world");

    scene.frame();

    assert_eq!(
        scene.border_box(on_one_line),
        rect(0.0, 0.0, 11.0 * CHARACTER_WIDTH, LINE_HEIGHT)
    );
    assert_eq!(
        scene.border_box(wrapped),
        rect(0.0, LINE_HEIGHT, 60.0, 2.0 * LINE_HEIGHT)
    );
    assert_eq!(
        scene.border_box(narrow),
        rect(0.0, LINE_HEIGHT, 60.0, 2.0 * LINE_HEIGHT)
    );
    assert!(
        scene
            .text_measurer
            .measured_texts
            .iter()
            .all(|text| text == "hello world")
    );
    assert!(!scene.text_measurer.measured_texts.is_empty());
}

#[test]
fn a_new_text_is_measured_again_and_moves_what_follows_it() {
    let mut scene = Scene::new(".line { display: flex; }");
    let line = scene.add(scene.root(), Role::Element, &["line"]);
    let label = scene.add_text(line, "short");
    let after = scene.add_text(line, "after");
    scene.frame();
    assert_eq!(scene.border_box(after), rect(50.0, 0.0, 50.0, 20.0));
    scene.text_measurer.measured_texts.clear();

    scene.tree.set_text(label, "a longer label");
    scene.frame();

    assert_eq!(scene.border_box(label), rect(0.0, 0.0, 140.0, 20.0));
    assert_eq!(scene.border_box(after), rect(140.0, 0.0, 50.0, 20.0));
    assert!(
        scene
            .text_measurer
            .measured_texts
            .iter()
            .all(|text| text == "a longer label"),
        "{:?}",
        scene.text_measurer.measured_texts
    );
}

#[test]
fn only_the_subtree_that_changed_is_laid_out_again() {
    let mut scene = Scene::new(
        ".columns { display: flex; }\n\
         .column { width: 200px; }\n\
         .wide { width: 300px; }",
    );
    let columns = scene.add(scene.root(), Role::Element, &["columns"]);
    let left = scene.add(columns, Role::Element, &["column"]);
    let left_texts = [
        scene.add_text(left, "left one"),
        scene.add_text(left, "left two is longer"),
    ];
    let right = scene.add(columns, Role::Element, &["column"]);
    let right_texts = [scene.add_text(right, "right one"), scene.add_text(right, "right two")];
    let first_frame = scene.frame();
    assert_eq!(first_frame.laid_out, 8);
    assert_eq!(scene.border_box(right_texts[1]), rect(200.0, 20.0, 200.0, 20.0));
    scene.text_measurer.measured_texts.clear();

    scene.add_classes(left, &["wide"]);
    let widened = scene.frame();

    assert_eq!(scene.border_box(left), rect(0.0, 0.0, 300.0, 40.0));
    assert_eq!(scene.border_box(left_texts[1]), rect(0.0, 20.0, 300.0, 20.0));
    assert_eq!(scene.border_box(right), rect(300.0, 0.0, 200.0, 40.0));
    assert_eq!(scene.border_box(right_texts[0]), rect(300.0, 0.0, 200.0, 20.0));
    assert_eq!(scene.border_box(right_texts[1]), rect(300.0, 20.0, 200.0, 20.0));
    assert!(
        scene
            .text_measurer
            .measured_texts
            .iter()
            .all(|text| text.starts_with("left")),
        "{:?}",
        scene.text_measurer.measured_texts
    );
    assert_eq!(
        widened.laid_out, 5,
        "the root, the columns, the widened column and its two texts"
    );
}

#[test]
fn a_frame_without_changes_lays_nothing_out() {
    let mut scene = Scene::new(".box { height: 10px; }");
    let element = scene.add(scene.root(), Role::Element, &["box"]);
    scene.add_text(element, "steady");
    scene.frame();
    scene.text_measurer.measured_texts.clear();

    let steady = scene.frame();

    assert_eq!(steady.resolved, 0);
    assert_eq!(steady.laid_out, 0);
    assert_eq!(steady.rebuilt, 0);
    assert!(scene.text_measurer.measured_texts.is_empty());
}

#[test]
fn a_change_that_only_paints_lays_nothing_out() {
    let mut scene = Scene::new(
        ".button { height: 28px; background-color: var(--color-surface); }\n\
         .button:hover { background-color: var(--color-surface-2); }",
    );
    let button = scene.add(scene.root(), Role::Element, &["button"]);
    scene.frame();

    scene.tree.set_state(button, States::HOVER, true);
    let hovered = scene.frame();

    assert_eq!(hovered.resolved, 1);
    assert_eq!(hovered.laid_out, 0);
}

#[test]
fn an_element_without_a_box_takes_no_space_until_it_is_shown_again() {
    let mut scene = Scene::new(
        ".row { height: 30px; }\n\
         .hidden { display: none; }",
    );
    let first = scene.add(scene.root(), Role::Element, &["row"]);
    let inside = scene.add(first, Role::Element, &["row"]);
    let second = scene.add(scene.root(), Role::Element, &["row"]);
    scene.frame();
    assert_eq!(scene.border_box(second), rect(0.0, 30.0, 800.0, 30.0));

    let hidden = scene.class("hidden");
    scene.tree.add_class(first, hidden);
    scene.frame();

    assert_eq!(scene.border_box(first), rect(0.0, 0.0, 0.0, 0.0));
    assert_eq!(scene.border_box(inside), rect(0.0, 0.0, 0.0, 0.0));
    assert_eq!(scene.border_box(second), rect(0.0, 0.0, 800.0, 30.0));

    scene.tree.remove_class(first, hidden);
    scene.frame();

    assert_eq!(scene.border_box(first), rect(0.0, 0.0, 800.0, 30.0));
    assert_eq!(scene.border_box(inside), rect(0.0, 0.0, 800.0, 30.0));
    assert_eq!(scene.border_box(second), rect(0.0, 30.0, 800.0, 30.0));
}

#[test]
fn children_are_laid_out_in_their_order_and_follow_every_change_of_it() {
    let mut scene = Scene::new(
        ".row { height: 30px; }\n\
         .tall { height: 50px; }",
    );
    let first = scene.add(scene.root(), Role::Element, &["row"]);
    let second = scene.add(scene.root(), Role::Element, &["row"]);
    scene.frame();

    let inserted = scene.tree.create(Role::Element);
    scene.add_classes(inserted, &["tall"]);
    scene.tree.insert_child(scene.root(), 0, inserted);
    scene.frame();
    assert_eq!(scene.tree.children(scene.root()), [inserted, first, second]);
    assert_eq!(scene.border_box(inserted), rect(0.0, 0.0, 800.0, 50.0));
    assert_eq!(scene.border_box(first), rect(0.0, 50.0, 800.0, 30.0));
    assert_eq!(scene.border_box(second), rect(0.0, 80.0, 800.0, 30.0));

    scene.tree.detach(first);
    scene.tree.append_child(scene.root(), first);
    scene.frame();
    assert_eq!(scene.border_box(second), rect(0.0, 50.0, 800.0, 30.0));
    assert_eq!(scene.border_box(first), rect(0.0, 80.0, 800.0, 30.0));

    scene.tree.remove(inserted);
    scene.frame();
    assert!(!scene.tree.contains(inserted));
    assert_eq!(scene.border_box(second), rect(0.0, 0.0, 800.0, 30.0));
    assert_eq!(scene.border_box(first), rect(0.0, 30.0, 800.0, 30.0));
}

#[test]
#[should_panic(expected = "styles are resolved before layout")]
fn laying_out_a_tree_whose_styles_are_not_resolved_panics() {
    let mut scene = Scene::new(".row { height: 30px; }");
    scene.add(scene.root(), Role::Element, &["row"]);

    scene.tree.compute_layout(scene.viewport, &mut scene.text_measurer);
}
