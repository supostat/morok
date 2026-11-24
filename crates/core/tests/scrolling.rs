mod support;

use std::time::Duration;

use morok_core::accesskit::{Action, ActionData, ActionRequest, Node, NodeId, Role as AccessibleRole, TreeId};
use morok_core::{
    DisplayItem, ElementId, ElementTree, Focusability, Key, Modifiers, Offset, Point, Role, ScrollBars, ScrollInput,
    ScrollPhase, Transform,
};
use support::{InputLog, Scene};

const STAGE: &str = ".viewport { overflow: auto; position: relative; width: 200px; height: 100px; }\n\
    .row { height: 40px; }\n\
    .wide { width: 500px; height: 10px; }\n\
    .endless { height: 30000000px; }\n\
    .section { height: 120px; }\n\
    .header { position: sticky; top: 0; height: 20px; }\n\
    .bar { position: absolute; left: 200px; top: 0; width: 8px; height: 100px; }\n\
    .thumb { width: 8px; }\n\
    .showing { opacity: 1; }";

/// How far apart two offsets may be and still count as one.
const TOLERANCE: f64 = 0.01;

fn assert_near(actual: impl Into<f64>, expected: impl Into<f64>, what: &str) {
    let (actual, expected) = (actual.into(), expected.into());
    assert!(
        (actual - expected).abs() < TOLERANCE,
        "{what}: {actual} is not {expected}"
    );
}

/// The transform the display list places the group of the element by: what the renderer
/// moves the element on the surface by.
fn painted_transform(tree: &ElementTree, element: ElementId) -> Transform {
    let laid_out = tree.border_box(element);
    tree.display_list()
        .items()
        .iter()
        .find_map(|item| match item {
            DisplayItem::PushGroup { effects } if effects.frame.border_box == laid_out => Some(effects.transform),
            _ => None,
        })
        .expect("the element paints as a group")
}

/// A viewport of 200 by 100 pixels at the corner of the surface that scrolls ten rows of 40.
struct ListScene {
    scene: Scene,
    viewport: ElementId,
    rows: Vec<ElementId>,
}

impl ListScene {
    fn new() -> Self {
        let mut scene = Scene::new(STAGE);
        let viewport = scene.add(scene.root(), Role::Scroll, &["viewport"]);
        let rows = (0..10).map(|_| scene.add(viewport, Role::Element, &["row"])).collect();
        scene.frame();
        Self { scene, viewport, rows }
    }

    fn offset(&self) -> Offset {
        self.scene.tree.scroll_offset(self.viewport)
    }

    fn scroll(&mut self, delta_y: f32, phase: ScrollPhase) {
        self.scene.tree.scroll_input(&ScrollInput {
            position: Point { x: 100.0, y: 50.0 },
            delta: Point { x: 0.0, y: delta_y },
            phase,
        });
    }

    fn node(&mut self, element: ElementId) -> Node {
        let id = NodeId::from(element);
        self.scene
            .tree
            .accessibility_tree()
            .nodes
            .into_iter()
            .find(|(node, _)| *node == id)
            .map(|(_, node)| node)
            .expect("the element has a node")
    }

    fn request(&mut self, action: Action, data: Option<ActionData>) {
        let request = ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: NodeId::from(self.viewport),
            data,
        };
        self.scene.tree.accessibility_action(&request, &mut InputLog::default());
    }
}

#[test]
fn the_limits_are_how_far_the_content_reaches_past_the_viewport_on_the_axes_that_scroll() {
    let mut list = ListScene::new();
    assert_eq!(list.scene.tree.scroll_limit(list.viewport), Offset { x: 0.0, y: 300.0 });

    list.scene.add(list.viewport, Role::Element, &["wide"]);
    list.scene.frame();

    assert_eq!(
        list.scene.tree.scroll_limit(list.viewport),
        Offset { x: 300.0, y: 310.0 }
    );
}

#[test]
fn an_offset_is_kept_within_the_limits_and_moves_the_content_with_no_layout_pass() {
    let mut list = ListScene::new();

    list.scene.tree.scroll_to(list.viewport, Offset { x: 0.0, y: 120.0 });
    let frame = list.scene.frame();

    assert_eq!(frame.laid_out_elements, 0, "a scroll lays nothing out");
    assert!(frame.rebuilt_elements > 0, "the content is painted where it shows now");
    assert_eq!(list.offset(), Offset { x: 0.0, y: 120.0 });
    assert_near(
        list.scene.tree.surface_box(list.rows[3]).y,
        0.0,
        "the fourth row is at the top",
    );
    assert_near(
        list.scene.tree.border_box(list.rows[3]).y,
        120.0,
        "layout keeps its boxes",
    );
    assert_eq!(
        list.scene.tree.element_at(Point { x: 10.0, y: 5.0 }),
        Some(list.rows[3])
    );

    list.scene.tree.scroll_to(list.viewport, Offset { x: 50.0, y: 1000.0 });
    list.scene.frame();
    assert_eq!(
        list.offset(),
        Offset { x: 0.0, y: 300.0 },
        "past the limits the offset stops at them"
    );
}

#[test]
fn a_notch_of_the_wheel_moves_the_content_under_the_pointer_and_stops_at_the_limits() {
    let mut list = ListScene::new();

    list.scroll(50.0, ScrollPhase::Discrete);
    assert_near(list.offset().y, 50.0, "the offset");
    list.scroll(1000.0, ScrollPhase::Discrete);
    assert_near(list.offset().y, 300.0, "the offset");
    list.scroll(-2000.0, ScrollPhase::Discrete);
    assert_near(list.offset().y, 0.0, "the offset");

    list.scene.tree.scroll_input(&ScrollInput {
        position: Point { x: 300.0, y: 50.0 },
        delta: Point { x: 0.0, y: 50.0 },
        phase: ScrollPhase::Discrete,
    });
    assert_near(list.offset().y, 0.0, "a scroll outside the viewport does not reach it");
}

#[test]
fn a_gesture_pulls_the_content_past_a_limit_against_resistance_and_the_spring_takes_it_back() {
    let mut list = ListScene::new();
    list.scroll(0.0, ScrollPhase::GestureBegan);
    list.scroll(-100.0, ScrollPhase::GestureChanged);
    let pulled = list.offset().y;
    assert!(pulled < 0.0 && pulled > -50.0, "the edge resists the pull: {pulled}");
    list.scroll(-100.0, ScrollPhase::GestureChanged);
    let pulled_further = list.offset().y;
    assert!(
        pulled_further < pulled && pulled - pulled_further < -pulled,
        "a further pull moves the content less"
    );

    list.scroll(0.0, ScrollPhase::GestureEnded);
    let start = list.scene.time;
    let first = list.scene.frame_at(start);
    assert_eq!(first.laid_out_elements, 0);
    assert_near(
        list.offset().y,
        pulled_further,
        "the spring starts where the gesture left the content",
    );
    list.scene.frame_at(start + Duration::from_millis(80));
    let returning = list.offset().y;
    assert!(
        returning > pulled_further && returning < 0.0,
        "the spring is on its way: {returning}"
    );
    let frame = list.scene.frame_at(start + Duration::from_secs(2));
    assert_near(list.offset().y, 0.0, "the spring brings the content to the limit");
    assert_eq!(frame.laid_out_elements, 0);
    list.scene.frame_at(start + Duration::from_secs(3));
    assert_near(list.offset().y, 0.0, "it stays there");
}

#[test]
fn momentum_after_a_gesture_stays_with_the_scroll_the_gesture_began_over() {
    let mut list = ListScene::new();
    list.scroll(0.0, ScrollPhase::GestureBegan);
    list.scroll(60.0, ScrollPhase::GestureChanged);
    list.scroll(0.0, ScrollPhase::GestureEnded);
    list.scene.tree.scroll_input(&ScrollInput {
        position: Point { x: 300.0, y: 50.0 },
        delta: Point { x: 0.0, y: 30.0 },
        phase: ScrollPhase::Momentum,
    });
    assert_near(
        list.offset().y,
        90.0,
        "momentum follows the gesture wherever the pointer went",
    );
    list.scroll(0.0, ScrollPhase::MomentumEnded);
    list.scene.frame();
    assert_near(list.offset().y, 90.0, "the offset");
}

#[test]
fn a_sticky_header_stays_at_the_top_of_the_scrollport_until_its_section_ends() {
    let mut scene = Scene::new(STAGE);
    let viewport = scene.add(scene.root(), Role::Scroll, &["viewport"]);
    let mut headers = Vec::new();
    for _ in 0..2 {
        let section = scene.add(viewport, Role::Element, &["section"]);
        headers.push(scene.add(section, Role::Element, &["header"]));
    }
    scene.frame();

    scene.tree.scroll_to(viewport, Offset { x: 0.0, y: 50.0 });
    let frame = scene.frame();
    assert_eq!(frame.laid_out_elements, 0);
    assert_near(
        scene.tree.surface_box(headers[0]).y,
        0.0,
        "the first header holds to the top",
    );
    assert_near(scene.tree.surface_box(headers[1]).y, 70.0, "the place on the surface");
    assert_eq!(scene.tree.element_at(Point { x: 10.0, y: 5.0 }), Some(headers[0]));

    scene.tree.scroll_to(viewport, Offset { x: 0.0, y: 110.0 });
    scene.frame();
    assert_near(
        scene.tree.surface_box(headers[0]).y,
        -10.0,
        "the end of its section pushes the header out",
    );
    assert_near(scene.tree.surface_box(headers[1]).y, 10.0, "the place on the surface");

    scene.tree.scroll_to(viewport, Offset::default());
    scene.frame();
    assert_near(
        scene.tree.surface_box(headers[0]).y,
        0.0,
        "at rest it is where layout put it",
    );
}

#[test]
fn keys_nothing_else_uses_scroll_the_scroll_around_focus() {
    let mut list = ListScene::new();
    let first_row = list.rows[0];
    list.scene.tree.set_focusability(first_row, Focusability::Focusable);
    assert!(list.scene.tree.focus(first_row));
    let mut log = InputLog::default();

    assert!(list.scene.press_key(Key::ArrowDown, Modifiers::NONE, &mut log));
    assert_near(list.offset().y, 40.0, "the offset");
    assert!(list.scene.press_key(Key::PageDown, Modifiers::NONE, &mut log));
    assert_near(list.offset().y, 127.5, "the offset");
    assert!(list.scene.press_key(Key::End, Modifiers::NONE, &mut log));
    assert_near(list.offset().y, 300.0, "the offset");
    assert!(
        !list.scene.press_key(Key::ArrowDown, Modifiers::NONE, &mut log),
        "at the limit the key moves nothing"
    );
    assert!(list.scene.press_key(Key::Space, Modifiers::SHIFT, &mut log));
    assert_near(list.offset().y, 212.5, "the offset");
    assert!(list.scene.press_key(Key::Home, Modifiers::NONE, &mut log));
    assert_near(list.offset().y, 0.0, "the offset");
}

#[test]
fn focus_the_keyboard_moves_is_scrolled_into_view() {
    let mut list = ListScene::new();
    let ninth_row = list.rows[8];
    list.scene.tree.set_focusability(ninth_row, Focusability::TabStop);

    assert!(list.scene.tree.focus(ninth_row));
    let frame = list.scene.frame();

    assert_eq!(frame.laid_out_elements, 0);
    assert_near(list.offset().y, 260.0, "the row is brought just into view");
    assert_near(
        list.scene.tree.surface_box(ninth_row).bottom(),
        100.0,
        "the place on the surface",
    );
}

#[test]
fn an_assistive_technology_reads_the_scroll_and_moves_it_by_a_page_or_to_an_offset() {
    let mut list = ListScene::new();
    list.scene
        .tree
        .set_accessible_role(list.viewport, Some(AccessibleRole::ScrollView));
    list.scene.frame();
    let node = list.node(list.viewport);
    assert_eq!((node.scroll_y(), node.scroll_y_max()), (Some(0.0), Some(300.0)));
    assert!(node.supports_action(Action::ScrollDown) && !node.supports_action(Action::ScrollRight));

    list.request(Action::ScrollDown, None);
    assert_near(list.offset().y, 87.5, "the offset");
    list.request(
        Action::SetScrollOffset,
        Some(ActionData::SetScrollOffset(morok_core::accesskit::Point {
            x: 0.0,
            y: 200.0,
        })),
    );
    assert_near(list.offset().y, 200.0, "the offset");
    list.scene.frame();
    assert_eq!(list.node(list.viewport).scroll_y(), Some(200.0));
}

#[test]
fn a_thumb_is_as_long_as_the_share_the_viewport_shows_and_travels_with_the_offset() {
    let mut list = ListScene::new();
    let bar = list.scene.add(list.scene.root(), Role::Element, &["bar"]);
    let thumb = list.scene.add(bar, Role::Element, &["thumb"]);
    let showing = list.scene.class("showing");
    list.scene.tree.set_scroll_bars(
        list.viewport,
        ScrollBars {
            vertical: Some(thumb),
            horizontal: None,
            showing,
        },
    );
    list.scene.frame();
    assert_near(
        list.scene.tree.border_box(thumb).height,
        25.0,
        "a quarter of the content shows",
    );
    assert_eq!(list.scene.tree.scroll_thumb_travel(thumb), Some(Offset::default()));

    list.scroll(150.0, ScrollPhase::Discrete);
    let frame = list.scene.frame();
    assert_eq!(
        frame.laid_out_elements, 0,
        "a thumb that keeps its length is not laid out"
    );
    assert_eq!(
        list.scene.tree.scroll_thumb_travel(thumb),
        Some(Offset { x: 0.0, y: 37.5 })
    );
    assert_near(list.scene.tree.surface_box(thumb).y, 37.5, "the place on the surface");
    assert!(
        list.scene.tree.classes(thumb).contains(&showing),
        "the thumb shows while the content moves"
    );

    list.scene.frame();
    list.scene.frame();
    assert!(
        !list.scene.tree.classes(thumb).contains(&showing),
        "it hides once the content rests"
    );
}

#[test]
fn small_scrolls_far_down_a_long_content_add_up() {
    let mut scene = Scene::new(STAGE);
    let viewport = scene.add(scene.root(), Role::Scroll, &["viewport"]);
    scene.add(viewport, Role::Element, &["endless"]);
    scene.frame();
    let far_down = 20_000_000.0;
    scene.tree.scroll_to(viewport, Offset { x: 0.0, y: far_down });
    scene.frame();
    assert_near(
        scene.tree.scroll_offset(viewport).y,
        far_down,
        "the content is scrolled far down",
    );
    let mut scroll = |delta_y: f32, phase: ScrollPhase| {
        scene.tree.scroll_input(&ScrollInput {
            position: Point { x: 100.0, y: 50.0 },
            delta: Point { x: 0.0, y: delta_y },
            phase,
        });
        scene.tree.scroll_offset(viewport).y
    };

    let mut offset = far_down;
    for _ in 0..8 {
        offset = scroll(0.5, ScrollPhase::Discrete);
    }
    assert_near(
        offset,
        far_down + 4.0,
        "eight turns of half a pixel, where an offset has steps of two",
    );

    scroll(0.5, ScrollPhase::GestureBegan);
    for _ in 0..7 {
        offset = scroll(0.5, ScrollPhase::GestureChanged);
    }
    assert_near(offset, far_down + 8.0, "and so do the moves of a gesture");
}

#[test]
fn an_offset_single_precision_cannot_hold_is_kept_and_one_pixel_scrolls_move_the_content_by_a_pixel() {
    let mut scene = Scene::new(STAGE);
    let viewport = scene.add(scene.root(), Role::Scroll, &["viewport"]);
    let row = scene.add(viewport, Role::Element, &["row"]);
    let section = scene.add(viewport, Role::Element, &["endless"]);
    let header = scene.add(section, Role::Element, &["header"]);
    let odd_far_down = 16_777_217.0;
    scene.tree.set_displacement(
        row,
        Offset {
            x: 0.0,
            y: odd_far_down + 30.0,
        },
    );
    scene.tree.scroll_to(
        viewport,
        Offset {
            x: 0.0,
            y: odd_far_down,
        },
    );
    scene.frame();

    assert_eq!(
        scene.tree.scroll_offset(viewport),
        Offset {
            x: 0.0,
            y: odd_far_down
        },
        "2^24 + 1, which single precision rounds to 2^24"
    );
    for scrolled in 0..4_u8 {
        assert_eq!(
            scene.tree.surface_box(row).y.to_bits(),
            (30.0 - f32::from(scrolled)).to_bits(),
            "the row after {scrolled} one-pixel scrolls"
        );
        assert_eq!(
            painted_transform(&scene.tree, row),
            Transform::translation(0.0, 30.0 - f64::from(scrolled)),
            "the row as the renderer is given it after {scrolled} one-pixel scrolls"
        );
        assert_eq!(
            scene.tree.surface_box(header).y.to_bits(),
            0.0_f32.to_bits(),
            "the sticky header of the section, laid out 40 down, after {scrolled} one-pixel scrolls"
        );
        assert_eq!(
            painted_transform(&scene.tree, header),
            Transform::translation(0.0, -40.0),
            "the header as the renderer is given it after {scrolled} one-pixel scrolls"
        );
        scene.tree.scroll_input(&ScrollInput {
            position: Point { x: 100.0, y: 50.0 },
            delta: Point { x: 0.0, y: 1.0 },
            phase: ScrollPhase::Discrete,
        });
        scene.frame();
    }
}

#[test]
fn an_element_a_component_moved_is_brought_into_view_where_it_stands() {
    let mut list = ListScene::new();
    let moved = list.rows[1];
    list.scene
        .tree
        .set_accessible_role(moved, Some(AccessibleRole::ListItem));
    list.scene.tree.set_displacement(moved, Offset { x: 0.0, y: 200.0 });
    list.scene.frame();

    let request = ActionRequest {
        action: Action::ScrollIntoView,
        target_tree: TreeId::ROOT,
        target_node: NodeId::from(moved),
        data: None,
    };
    list.scene.tree.accessibility_action(&request, &mut InputLog::default());
    list.scene.frame();

    assert_near(
        list.offset().y,
        180.0,
        "the row laid out from 40 to 80 stands from 240 to 280, and the viewport shows 100",
    );
}
