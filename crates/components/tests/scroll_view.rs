mod support;

use std::borrow::Cow;

use morok_components::ScrollView;
use morok_components::styles::CLASSES;
use morok_components::view::Mount;
use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{ElementId, Point, ScrollInput, ScrollPhase};
use morok_style::{Declaration, Display, FlexDirection, LengthPercentage, LengthPercentageAuto, Pixels};
use support::Stage;

fn pixels(length: f32) -> LengthPercentageAuto {
    LengthPercentageAuto::LengthPercentage(LengthPercentage {
        pixels: Pixels(length),
        fraction: None,
        tokens: Cow::Borrowed(&[]),
    })
}

/// A column of eight rows 50 pixels high.
fn rows(mount: &mut Mount<'_>) -> ElementId {
    let rows: Vec<ElementId> = (0..8)
        .map(|_| mount.element().inline(Declaration::Height(pixels(50.0))).id())
        .collect();
    let mut column = mount.element();
    column
        .inline(Declaration::Display(Display::Flex))
        .inline(Declaration::FlexDirection(FlexDirection::Column));
    for row in rows {
        column.child(row);
    }
    column.id()
}

/// A view of 200 by 100 pixels over the rows.
fn stage_with_view() -> (Stage, ElementId) {
    let mut stage = Stage::new();
    let view = stage.mount(|mount| {
        let view = ScrollView::new(rows).label("Files").mount(mount);
        mount
            .build(view)
            .inline(Declaration::Width(pixels(200.0)))
            .inline(Declaration::Height(pixels(100.0)));
        view
    });
    stage.frame();
    (stage, view)
}

#[test]
fn a_scroll_view_is_a_viewport_with_a_bar_of_each_axis_whose_thumbs_the_tree_moves() {
    let (mut stage, view) = stage_with_view();
    let [viewport, vertical, horizontal] = stage.children(view)[..] else {
        panic!("a scroll view holds its viewport and two bars");
    };
    let tree = stage.tree();
    assert_eq!(tree.classes(view), [CLASSES.scroll_view]);
    assert_eq!(tree.classes(viewport), [CLASSES.scroll_viewport]);
    assert_eq!(tree.classes(vertical), [CLASSES.scroll_bar, CLASSES.vertical]);
    assert_eq!(tree.classes(horizontal), [CLASSES.scroll_bar, CLASSES.horizontal]);
    let bars = tree.scroll_bars(viewport).expect("the view gives its viewport bars");
    assert_eq!(bars.vertical, Some(tree.children(vertical)[0]));
    assert_eq!(bars.horizontal, Some(tree.children(horizontal)[0]));
    assert_eq!(tree.scroll_limit(viewport), Point { x: 0.0, y: 300.0 });
    let node = stage.accessible_node(viewport);
    assert_eq!((node.role(), node.label()), (AccessibleRole::ScrollView, Some("Files")));
}

#[test]
fn the_wheel_over_the_view_scrolls_its_content_and_shows_the_thumb_while_it_moves() {
    let (mut stage, view) = stage_with_view();
    let viewport = stage.children(view)[0];
    let thumb = stage
        .tree()
        .scroll_bars(viewport)
        .and_then(|bars| bars.vertical)
        .expect("the view has a vertical thumb");

    stage.surfaces.surface_mut(stage.surface).scroll_input(&ScrollInput {
        position: Point { x: 100.0, y: 50.0 },
        delta: Point { x: 0.0, y: 150.0 },
        phase: ScrollPhase::Discrete,
    });
    stage.frame();

    assert_eq!(stage.tree().scroll_offset(viewport), Point { x: 0.0, y: 150.0 });
    assert!(stage.tree().classes(thumb).contains(&CLASSES.showing));
    stage.frame();
    stage.frame();
    assert!(!stage.tree().classes(thumb).contains(&CLASSES.showing));
}
