mod support;

use morok_core::{Offset, Point, Rect, Role};
use support::Scene;

const STAGE: &str = ".port { overflow: hidden; width: 100px; height: 100px; }\n\
    .card { width: 40px; height: 20px; }\n\
    .far { margin-left: 80px; }\n\
    .pin { position: relative; width: 30px; height: 20px; }\n\
    .row { position: relative; overflow: hidden; width: 200px; height: 20px; }\n\
    .cell { position: absolute; top: 0; left: 0; width: 100px; height: 20px; }\n\
    .filling { position: absolute; top: 0; right: 0; bottom: 0; left: 0; padding: 0 12px; }\n\
    .anchored { position: absolute; top: 0; bottom: 0; left: 0; }\n\
    .label { height: 20px; }";

#[test]
fn a_displaced_element_moves_as_a_picture_with_no_style_resolved_and_no_box_laid_out() {
    let mut scene = Scene::new(STAGE);
    let card = scene.add(scene.root(), Role::Element, &["card"]);
    let inside = scene.add(card, Role::Element, &["card"]);
    scene.frame();
    assert_eq!(scene.tree.displacement(card), None);

    scene.tree.set_displacement(card, Offset { x: 30.0, y: 50.0 });
    let frame = scene.frame();

    assert_eq!((frame.styled_elements, frame.laid_out_elements), (0, 0));
    assert!(frame.rebuilt_elements > 0, "the picture is drawn at its new place");
    assert_eq!(
        scene.border_box(card),
        Rect::new(0.0, 0.0, 40.0, 20.0),
        "layout knows nothing of it"
    );
    assert_eq!(scene.tree.surface_box(card), Rect::new(30.0, 50.0, 40.0, 20.0));
    assert_eq!(
        scene.tree.surface_box(inside),
        Rect::new(30.0, 50.0, 40.0, 20.0),
        "what is inside moves with it"
    );
    assert_eq!(scene.tree.element_at(Point { x: 35.0, y: 55.0 }), Some(inside));
    assert_eq!(scene.tree.element_at(Point { x: 5.0, y: 5.0 }), Some(scene.root()));
}

#[test]
fn a_displacement_set_again_to_what_it_is_asks_for_no_frame_work() {
    let mut scene = Scene::new(STAGE);
    let card = scene.add(scene.root(), Role::Element, &["card"]);
    scene.tree.set_displacement(card, Offset { x: 30.0, y: 50.0 });
    scene.frame();

    scene.tree.set_displacement(card, Offset { x: 30.0, y: 50.0 });
    let frame = scene.frame();

    assert_eq!(
        (frame.styled_elements, frame.laid_out_elements, frame.rebuilt_elements),
        (0, 0, 0)
    );
}

#[test]
fn what_is_positioned_inside_a_displaced_element_is_clipped_where_the_surface_shows_it() {
    let mut scene = Scene::new(STAGE);
    let port = scene.add(scene.root(), Role::Element, &["port"]);
    let card = scene.add(port, Role::Element, &["card", "far"]);
    let pin = scene.add(card, Role::Element, &["pin"]);
    scene.tree.set_displacement(card, Offset { x: -60.0, y: 0.0 });
    scene.frame();

    assert_eq!(scene.tree.surface_box(pin), Rect::new(20.0, 0.0, 30.0, 20.0));
    assert_eq!(
        scene.tree.element_at(Point { x: 45.0, y: 10.0 }),
        Some(pin),
        "the port clips where it stands, not where the displacement would take it"
    );

    scene.tree.set_displacement(card, Offset { x: 10.0, y: 0.0 });
    scene.frame();
    assert_eq!(
        scene.tree.element_at(Point { x: 105.0, y: 10.0 }),
        Some(scene.root()),
        "past the edge of the port the pin is clipped"
    );
    assert_eq!(scene.tree.element_at(Point { x: 95.0, y: 10.0 }), Some(pin));
}

#[test]
fn a_text_of_a_fixed_size_under_a_parent_that_hides_overflow_changes_without_a_box_laid_out() {
    let mut scene = Scene::new(STAGE);
    let row = scene.add(scene.root(), Role::Element, &["row"]);
    let cell = scene.add(row, Role::Text, &["cell"]);
    scene.tree.set_text(cell, "first");
    scene.frame();
    let line_before = scene.tree.line_boxes(cell)[0].rect;

    scene.tree.set_text(cell, "another text");
    let frame = scene.frame();

    assert_eq!(
        frame.laid_out_elements, 0,
        "the box of the cell is the same whatever it says"
    );
    assert!(frame.rebuilt_elements > 0);
    let line = scene.tree.line_boxes(cell)[0].rect;
    assert!(line.width > line_before.width, "the line is set again: {line:?}");
    assert_eq!(scene.border_box(cell), Rect::new(0.0, 0.0, 100.0, 20.0));
}

#[test]
fn a_text_its_insets_fill_under_a_parent_that_hides_overflow_changes_without_a_box_laid_out() {
    let mut scene = Scene::new(STAGE);
    let row = scene.add(scene.root(), Role::Element, &["row"]);
    let label = scene.add(row, Role::Text, &["filling"]);
    scene.tree.set_text(label, "first");
    scene.frame();

    scene.tree.set_text(label, "another text");
    let frame = scene.frame();

    assert_eq!(
        frame.laid_out_elements, 0,
        "the insets set the box of the label whatever it says"
    );
    assert!(frame.rebuilt_elements > 0);
    assert_eq!(scene.border_box(label), Rect::new(0.0, 0.0, 200.0, 20.0));
}

#[test]
fn a_text_placed_by_one_inset_of_an_axis_is_laid_out_when_it_changes() {
    let mut scene = Scene::new(STAGE);
    let row = scene.add(scene.root(), Role::Element, &["row"]);
    let label = scene.add(row, Role::Text, &["anchored"]);
    scene.tree.set_text(label, "first");
    scene.frame();

    scene.tree.set_text(label, "another text");
    let frame = scene.frame();

    assert!(
        frame.laid_out_elements > 0,
        "with no right inset its width follows its text"
    );
}

#[test]
fn a_text_as_wide_as_what_it_says_is_laid_out_when_it_changes() {
    let mut scene = Scene::new(STAGE);
    let row = scene.add(scene.root(), Role::Element, &["row"]);
    let label = scene.add(row, Role::Text, &["label"]);
    scene.tree.set_text(label, "first");
    scene.frame();

    scene.tree.set_text(label, "another text");
    let frame = scene.frame();

    assert!(frame.laid_out_elements > 0, "its width follows its text");
}
