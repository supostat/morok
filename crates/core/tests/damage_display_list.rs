mod support;

use morok_core::{BoxFrame, Corners, DisplayItem, ElementId, GradientShape, GradientStop, Point, Rect, Role, Sides};
use morok_css::SourceFile;
use morok_style::{Color, Pixels, ShadowLayer, States};
use support::Scene;

const TRANSPARENT: Color = Color {
    red: 0,
    green: 0,
    blue: 0,
    alpha: 0.0,
};

/// A display item with the stops of its gradient, so that two display lists can be compared.
#[derive(Clone, Debug, PartialEq)]
struct PaintedItem {
    item: DisplayItem,
    gradient_stops: Vec<GradientStop>,
}

fn painted_items(scene: &Scene) -> Vec<PaintedItem> {
    let display_list = scene.tree.display_list();
    display_list
        .items()
        .iter()
        .map(|item| match *item {
            DisplayItem::Gradient { frame, mut gradient } => {
                let gradient_stops = display_list.gradient_stops(gradient.stops).to_vec();
                gradient.stops.end -= gradient.stops.start;
                gradient.stops.start = 0;
                PaintedItem {
                    item: DisplayItem::Gradient { frame, gradient },
                    gradient_stops,
                }
            }
            other => PaintedItem {
                item: other,
                gradient_stops: Vec::new(),
            },
        })
        .collect()
}

/// What each item of the display list is, with the border box it belongs to.
fn item_kinds(scene: &Scene) -> Vec<(&'static str, Rect)> {
    let mut kinds = Vec::new();
    for item in scene.tree.display_list().items() {
        kinds.push(match item {
            DisplayItem::Shadow { frame, layer } if layer.inset => ("inset shadow", frame.border_box),
            DisplayItem::Shadow { frame, .. } => ("shadow", frame.border_box),
            DisplayItem::Background { frame, .. } => ("background", frame.border_box),
            DisplayItem::Gradient { frame, .. } => ("gradient", frame.border_box),
            DisplayItem::Border { frame, .. } => ("border", frame.border_box),
            DisplayItem::Outline { frame, .. } => ("outline", frame.border_box),
            DisplayItem::PushClip { frame } => ("push clip", frame.border_box),
            DisplayItem::GlyphRun { origin, .. } => ("glyph run", Rect::new(origin.x, origin.y, 0.0, 0.0)),
            DisplayItem::TextDecoration { rect, .. } => ("text decoration", *rect),
            DisplayItem::PopClip => ("pop clip", Rect::ZERO),
            DisplayItem::PushGroup { effects } => ("push group", effects.frame.border_box),
            DisplayItem::PopGroup => ("pop group", Rect::ZERO),
        });
    }
    kinds
}

fn background_color_of(scene: &Scene, element: ElementId) -> Color {
    let border_box = scene.border_box(element);
    scene
        .tree
        .display_list()
        .items()
        .iter()
        .find_map(|item| match item {
            DisplayItem::Background { frame, color } if frame.border_box == border_box => Some(*color),
            _ => None,
        })
        .expect("the element has a background item")
}

fn damage(scene: &Scene) -> &[Rect] {
    scene.tree.damage().rectangles()
}

fn damage_bounds(scene: &Scene) -> Rect {
    let (first, rest) = damage(scene).split_first().expect("the frame damaged something");
    rest.iter().fold(*first, |bounds, damaged| bounds.union(damaged))
}

fn uniform_radii(radius: f32) -> Corners<f32> {
    Corners {
        top_left: radius,
        top_right: radius,
        bottom_right: radius,
        bottom_left: radius,
    }
}

const TOOLBAR: &str = ".app { display: flex; gap: 20px; padding: 20px; background-color: var(--color-bg); }\n\
    .button { width: 100px; height: 30px; background-color: var(--color-surface); \
    border: 1px solid var(--color-line); }\n\
    .button:hover { background-color: var(--color-surface-2); }\n\
    .button:focus-visible { outline: 2px solid var(--color-focus); outline-offset: 2px; }\n\
    .raised { box-shadow: var(--shadow-float); }\n\
    .tall { height: 50px; }\n\
    .hidden { display: none; }";

fn toolbar() -> (Scene, [ElementId; 3]) {
    let mut scene = Scene::new(TOOLBAR);
    scene.add_classes(scene.root(), &["app"]);
    let buttons = [(); 3].map(|()| scene.add(scene.root(), Role::Element, &["button"]));
    (scene, buttons)
}

#[test]
fn the_first_display_list_holds_every_box_and_damages_the_whole_surface() {
    let (mut scene, buttons) = toolbar();

    let first_frame = scene.frame();

    assert_eq!(first_frame.rebuilt, 4);
    let root_box = Rect::new(0.0, 0.0, 800.0, 600.0);
    assert_eq!(
        item_kinds(&scene),
        [
            ("background", root_box),
            ("background", scene.border_box(buttons[0])),
            ("border", scene.border_box(buttons[0])),
            ("background", scene.border_box(buttons[1])),
            ("border", scene.border_box(buttons[1])),
            ("background", scene.border_box(buttons[2])),
            ("border", scene.border_box(buttons[2])),
        ]
    );
    assert_eq!(damage(&scene), [root_box]);
}

#[test]
fn a_state_flip_damages_only_its_rectangle_and_the_display_list_changes_only_inside_it() {
    let (mut scene, buttons) = toolbar();
    scene.frame();
    let before = painted_items(&scene);
    let hovered_box = scene.border_box(buttons[1]);
    assert_eq!(hovered_box, Rect::new(140.0, 20.0, 100.0, 30.0));

    scene.tree.set_state(buttons[1], States::HOVER, true);
    let hovered = scene.frame();

    assert_eq!(hovered.rebuilt, 1);
    assert_eq!(damage(&scene), [hovered_box]);
    let after = painted_items(&scene);
    assert_eq!(after.len(), before.len());
    let changed: Vec<_> = (0..after.len())
        .filter(|position| after[*position] != before[*position])
        .collect();
    assert_eq!(changed, [3], "only the background of the hovered button changed");
    assert_eq!(
        after[3].item,
        DisplayItem::Background {
            frame: BoxFrame {
                border_box: hovered_box,
                border_widths: Sides::uniform(1.0),
                radii: uniform_radii(0.0),
            },
            color: scene.light_color("color-surface-2"),
        }
    );
}

#[test]
fn a_frame_without_changes_damages_nothing_and_keeps_the_display_list() {
    let (mut scene, buttons) = toolbar();
    scene.tree.set_state(buttons[0], States::HOVER, true);
    scene.frame();
    let before = painted_items(&scene);

    scene.tree.set_state(buttons[0], States::HOVER, true);
    let steady = scene.frame();

    assert_eq!(steady.rebuilt, 0);
    assert!(scene.tree.damage().is_empty());
    assert_eq!(painted_items(&scene), before);
}

#[test]
fn an_outline_and_a_shadow_damage_everything_they_reach() {
    let (mut scene, buttons) = toolbar();
    scene.frame();
    let border_box = scene.border_box(buttons[0]);

    scene.tree.set_state(buttons[0], States::FOCUS_VISIBLE, true);
    scene.frame();

    assert_eq!(damage(&scene), [border_box.expanded(Sides::uniform(4.0))]);
    assert_eq!(
        scene.tree.display_list().items()[3],
        DisplayItem::Outline {
            frame: BoxFrame {
                border_box,
                border_widths: Sides::uniform(1.0),
                radii: uniform_radii(0.0),
            },
            width: 2.0,
            offset: 2.0,
            color: scene.light_color("color-focus"),
        },
        "the ring of a flex item is painted with the item, before the items after it"
    );

    scene.tree.set_state(buttons[0], States::FOCUS_VISIBLE, false);
    scene.frame();
    assert_eq!(
        damage(&scene),
        [border_box.expanded(Sides::uniform(4.0))],
        "the ring that went away is damaged as well"
    );

    scene.add_classes(buttons[0], &["raised"]);
    scene.frame();

    let shadow_reach = Sides {
        top: 36.0 - 8.0,
        right: 36.0,
        bottom: 36.0 + 8.0,
        left: 36.0,
    };
    assert_eq!(damage(&scene), [border_box.expanded(shadow_reach)]);
}

#[test]
fn the_primitives_of_a_box_are_listed_bottom_to_top() {
    let mut scene = Scene::new(
        ".box { width: 200px; height: 100px; margin: 50px; color: var(--color-text); \
         background-color: var(--color-surface); \
         background-image: linear-gradient(to right, var(--color-accent), transparent 50%, currentColor); \
         border: 2px solid var(--color-line); border-radius: var(--radius-lg) 0; \
         box-shadow: var(--shadow-float); outline: 1px solid var(--color-focus); outline-offset: 3px; }",
    );
    let element = scene.add(scene.root(), Role::Element, &["box"]);

    scene.frame();

    let frame = BoxFrame {
        border_box: Rect::new(50.0, 50.0, 200.0, 100.0),
        border_widths: Sides::uniform(2.0),
        radii: Corners {
            top_left: 12.0,
            top_right: 0.0,
            bottom_right: 12.0,
            bottom_left: 0.0,
        },
    };
    assert_eq!(scene.border_box(element), frame.border_box);
    assert_eq!(frame.padding_box(), Rect::new(52.0, 52.0, 196.0, 96.0));
    let shadow_color = |alpha: f32| Color {
        red: 17,
        green: 19,
        blue: 24,
        alpha,
    };
    let items = painted_items(&scene);
    let listed: Vec<_> = items.iter().map(|painted| painted.item).collect();
    let DisplayItem::Gradient { gradient, .. } = listed[3] else {
        panic!("the fourth item is the gradient: {:?}", listed[3]);
    };
    assert_eq!(
        listed,
        [
            DisplayItem::Shadow {
                frame,
                layer: ShadowLayer {
                    offset_x: Pixels(0.0),
                    offset_y: Pixels(1.0),
                    blur: Pixels(3.0),
                    spread: Pixels(0.0),
                    color: shadow_color(0.10),
                    inset: false,
                },
            },
            DisplayItem::Shadow {
                frame,
                layer: ShadowLayer {
                    offset_x: Pixels(0.0),
                    offset_y: Pixels(8.0),
                    blur: Pixels(24.0),
                    spread: Pixels(0.0),
                    color: shadow_color(0.14),
                    inset: false,
                },
            },
            DisplayItem::Background {
                frame,
                color: scene.light_color("color-surface"),
            },
            DisplayItem::Gradient { frame, gradient },
            DisplayItem::Border {
                frame,
                colors: Sides::uniform(scene.light_color("color-line")),
            },
            DisplayItem::Outline {
                frame,
                width: 1.0,
                offset: 3.0,
                color: scene.light_color("color-focus"),
            },
        ]
    );
    assert_eq!(gradient.shape, GradientShape::Linear { angle: 90.0 });
    assert_eq!(
        items[3].gradient_stops,
        [
            GradientStop {
                offset: 0.0,
                color: scene.light_color("color-accent"),
            },
            GradientStop {
                offset: 0.5,
                color: TRANSPARENT,
            },
            GradientStop {
                offset: 1.0,
                color: scene.light_color("color-text"),
            },
        ]
    );
}

#[test]
fn a_radial_gradient_is_resolved_against_the_padding_box() {
    let mut scene = Scene::new(
        ".spot { width: 200px; height: 100px; margin: 10px; border: 5px solid transparent; \
         color: var(--color-text); \
         background-image: radial-gradient(circle closest-side at 25% 50%, var(--color-accent), currentColor 9px); }",
    );
    scene.add(scene.root(), Role::Element, &["spot"]);

    scene.frame();

    let items = painted_items(&scene);
    assert_eq!(items.len(), 1, "a transparent border draws nothing: {items:?}");
    let DisplayItem::Gradient { gradient, frame } = items[0].item else {
        panic!("the item is the gradient: {:?}", items[0].item);
    };
    assert_eq!(frame.padding_box(), Rect::new(15.0, 15.0, 190.0, 90.0));
    assert_eq!(
        gradient.shape,
        GradientShape::Radial {
            center: Point { x: 62.5, y: 60.0 },
            horizontal_radius: 45.0,
            vertical_radius: 45.0,
        }
    );
    assert_eq!(
        items[0].gradient_stops,
        [
            GradientStop {
                offset: 0.0,
                color: scene.light_color("color-accent"),
            },
            GradientStop {
                offset: 0.2,
                color: scene.light_color("color-text"),
            },
        ]
    );
}

#[test]
fn inset_shadows_lie_over_the_background_and_under_the_border() {
    let tokens = SourceFile::new(
        "tokens.css",
        ":root { --color-fill: #FFFFFF; --color-edge: #000000; \
         --shadow-well: inset 0 1px 2px rgba(0, 0, 0, 0.5), 0 4px 8px rgba(0, 0, 0, 0.25), inset 0 0 0 1px #FF0000; }\n\
         @media (prefers-color-scheme: dark) { :root { --color-fill: #000000; --color-edge: #FFFFFF; \
         --shadow-well: inset 0 1px 2px rgba(0, 0, 0, 0.5), 0 4px 8px rgba(0, 0, 0, 0.25), inset 0 0 0 1px #FF0000; } }\n",
    );
    let mut scene = Scene::with_tokens(
        &tokens,
        ".well { width: 100px; height: 40px; background-color: var(--color-fill); \
         border: 1px solid var(--color-edge); box-shadow: var(--shadow-well); }",
    );
    let well = scene.add(scene.root(), Role::Element, &["well"]);

    scene.frame();

    let border_box = scene.border_box(well);
    assert_eq!(
        item_kinds(&scene),
        [
            ("shadow", border_box),
            ("background", border_box),
            ("inset shadow", border_box),
            ("inset shadow", border_box),
            ("border", border_box),
        ]
    );
    let spreads: Vec<_> = scene
        .tree
        .display_list()
        .items()
        .iter()
        .filter_map(|item| match item {
            DisplayItem::Shadow { layer, .. } if layer.inset => Some(layer.spread),
            _ => None,
        })
        .collect();
    assert_eq!(
        spreads,
        [Pixels(1.0), Pixels(0.0)],
        "the first layer of the token is the topmost"
    );
}

#[test]
fn radii_larger_than_the_box_are_fitted_to_it() {
    let mut scene = Scene::new(
        ".pill { width: 100px; height: 40px; border-radius: var(--radius-full); \
         background-color: var(--color-accent); }",
    );
    scene.add(scene.root(), Role::Element, &["pill"]);

    scene.frame();

    let DisplayItem::Background { frame, .. } = scene.tree.display_list().items()[0] else {
        panic!("the pill has a background");
    };
    assert_eq!(frame.radii, uniform_radii(20.0));
}

#[test]
fn an_element_that_hides_its_overflow_clips_the_items_of_its_content() {
    let mut scene = Scene::new(
        ".viewport { overflow: hidden; width: 100px; height: 50px; border: 1px solid var(--color-line); \
         border-radius: var(--radius-md); outline: 1px solid var(--color-focus); }\n\
         .content { height: 200px; background-color: var(--color-surface); }\n\
         .empty { overflow: hidden; height: 10px; }",
    );
    let viewport = scene.add(scene.root(), Role::Element, &["viewport"]);
    let content = scene.add(viewport, Role::Element, &["content"]);
    scene.add(scene.root(), Role::Element, &["empty"]);

    scene.frame();

    let viewport_box = scene.border_box(viewport);
    assert_eq!(
        item_kinds(&scene),
        [
            ("border", viewport_box),
            ("push clip", viewport_box),
            ("background", scene.border_box(content)),
            ("pop clip", Rect::ZERO),
            ("outline", viewport_box),
        ]
    );
    assert_eq!(
        scene.tree.display_list().items()[1],
        DisplayItem::PushClip {
            frame: BoxFrame {
                border_box: viewport_box,
                border_widths: Sides::uniform(1.0),
                radii: uniform_radii(8.0),
            },
        }
    );
}

#[test]
fn the_outline_of_a_block_lies_over_the_block_after_it() {
    let mut scene = Scene::new(
        ".row { height: 30px; background-color: var(--color-surface); }\n\
         .focused { outline: 2px solid var(--color-focus); }",
    );
    let focused = scene.add(scene.root(), Role::Element, &["row", "focused"]);
    let nested = scene.add(focused, Role::Element, &["row", "focused"]);
    let after = scene.add(scene.root(), Role::Element, &["row"]);

    scene.frame();

    assert_eq!(
        item_kinds(&scene),
        [
            ("background", scene.border_box(focused)),
            ("background", scene.border_box(nested)),
            ("background", scene.border_box(after)),
            ("outline", scene.border_box(nested)),
            ("outline", scene.border_box(focused)),
        ]
    );
}

#[test]
fn the_items_of_a_flex_container_paint_one_after_another() {
    let mut scene = Scene::new(
        ".bar { display: flex; }\n\
         .item { width: 50px; height: 30px; background-color: var(--color-surface); }\n\
         .focused { outline: 2px solid var(--color-focus); }\n\
         .after { height: 10px; background-color: var(--color-bg); }",
    );
    let bar = scene.add(scene.root(), Role::Element, &["bar"]);
    let focused = scene.add(bar, Role::Element, &["item", "focused"]);
    let inside = scene.add(focused, Role::Element, &["item"]);
    let next = scene.add(bar, Role::Element, &["item"]);
    let after = scene.add(scene.root(), Role::Element, &["after"]);

    scene.frame();

    assert_eq!(
        item_kinds(&scene),
        [
            ("background", scene.border_box(after)),
            ("background", scene.border_box(focused)),
            ("background", scene.border_box(inside)),
            ("outline", scene.border_box(focused)),
            ("background", scene.border_box(next)),
        ],
        "the block after the bar paints its background before the items of the bar"
    );
}

#[test]
fn positioned_elements_paint_over_the_flow_in_tree_order() {
    let mut scene = Scene::new(
        ".row { height: 30px; background-color: var(--color-surface); }\n\
         .badge { position: absolute; top: 0; left: 0; width: 10px; height: 10px; \
         background-color: var(--color-danger); }\n\
         .lifted { position: relative; }",
    );
    let holder = scene.add(scene.root(), Role::Element, &["row"]);
    let badge = scene.add(holder, Role::Element, &["badge"]);
    let plain = scene.add(scene.root(), Role::Element, &["row"]);
    let lifted = scene.add(scene.root(), Role::Element, &["row", "lifted"]);
    let last = scene.add(scene.root(), Role::Element, &["row"]);

    scene.frame();

    assert_eq!(
        item_kinds(&scene),
        [
            ("background", scene.border_box(holder)),
            ("background", scene.border_box(plain)),
            ("background", scene.border_box(last)),
            ("background", scene.border_box(badge)),
            ("background", scene.border_box(lifted)),
        ]
    );
}

#[test]
fn a_positioned_element_is_clipped_only_by_the_ancestors_that_contain_it() {
    let mut scene = Scene::new(
        ".scroller { overflow: hidden; height: 50px; }\n\
         .anchor { position: relative; overflow: hidden; height: 50px; }\n\
         .popup { position: absolute; top: 0; left: 0; width: 10px; height: 10px; \
         background-color: var(--color-danger); }\n\
         .lifted { position: relative; height: 10px; background-color: var(--color-accent); }",
    );
    let scroller = scene.add(scene.root(), Role::Element, &["scroller"]);
    let escaping = scene.add(scroller, Role::Element, &["popup"]);
    let lifted = scene.add(scroller, Role::Element, &["lifted"]);
    let anchor = scene.add(scene.root(), Role::Element, &["anchor"]);
    let contained = scene.add(anchor, Role::Element, &["popup"]);

    scene.frame();

    assert_eq!(scene.border_box(escaping), Rect::new(0.0, 0.0, 10.0, 10.0));
    assert_eq!(scene.border_box(contained), Rect::new(0.0, 50.0, 10.0, 10.0));
    assert_eq!(
        item_kinds(&scene),
        [
            ("background", scene.border_box(escaping)),
            ("push clip", scene.border_box(scroller)),
            ("background", scene.border_box(lifted)),
            ("pop clip", Rect::ZERO),
            ("push clip", scene.border_box(anchor)),
            ("background", scene.border_box(contained)),
            ("pop clip", Rect::ZERO),
        ]
    );
}

#[test]
fn an_element_that_moves_damages_where_it_was_and_where_it_is() {
    let mut scene = Scene::new(
        ".app { display: flex; }\n\
         .list { width: 300px; height: 300px; }\n\
         .row { height: 30px; background-color: var(--color-surface); }\n\
         .tall { height: 50px; }\n\
         .aside { width: 100px; height: 300px; background-color: var(--color-bg); }",
    );
    scene.add_classes(scene.root(), &["app"]);
    let list = scene.add(scene.root(), Role::Element, &["list"]);
    let rows = [(); 3].map(|()| scene.add(list, Role::Element, &["row"]));
    let aside = scene.add(scene.root(), Role::Element, &["aside"]);
    scene.frame();
    let before = painted_items(&scene);

    scene.add_classes(rows[0], &["tall"]);
    let grown = scene.frame();

    assert_eq!(grown.rebuilt, 3);
    assert_eq!(scene.border_box(rows[0]), Rect::new(0.0, 0.0, 300.0, 50.0));
    assert_eq!(scene.border_box(rows[2]), Rect::new(0.0, 80.0, 300.0, 30.0));
    assert_eq!(damage_bounds(&scene), Rect::new(0.0, 0.0, 300.0, 110.0));
    let aside_box = scene.border_box(aside);
    assert!(damage(&scene).iter().all(|damaged| !damaged.intersects(&aside_box)));
    let after = painted_items(&scene);
    assert_eq!(after[3], before[3], "the aside is taken over as it was");
    assert_ne!(after[..3], before[..3]);
}

#[test]
fn an_element_that_leaves_the_tree_damages_what_it_drew() {
    let (mut scene, buttons) = toolbar();
    scene.add_classes(buttons[2], &["raised"]);
    scene.frame();
    let last_box = scene.border_box(buttons[2]);
    let last_bounds = last_box.expanded(Sides {
        top: 28.0,
        right: 36.0,
        bottom: 44.0,
        left: 36.0,
    });

    scene.tree.remove(buttons[2]);
    let removed = scene.frame();

    assert_eq!(removed.rebuilt, 0);
    assert_eq!(damage(&scene), [last_bounds]);
    assert_eq!(item_kinds(&scene).len(), 5);

    let middle_box = scene.border_box(buttons[1]);
    scene.tree.detach(buttons[1]);
    scene.frame();
    assert_eq!(damage(&scene), [middle_box]);
    assert_eq!(item_kinds(&scene).len(), 3);

    scene.tree.append_child(scene.root(), buttons[1]);
    let returned = scene.frame();
    assert_eq!(returned.rebuilt, 1);
    assert_eq!(damage(&scene), [middle_box]);
    assert_eq!(item_kinds(&scene).len(), 5);
}

#[test]
fn an_element_without_a_box_leaves_the_display_list_with_its_descendants() {
    let (mut scene, buttons) = toolbar();
    let inside = scene.add(buttons[0], Role::Element, &["button"]);
    scene.frame();
    let first_box = scene.border_box(buttons[0]);
    let inside_box = scene.border_box(inside);
    let second_box = scene.border_box(buttons[1]);
    assert_eq!(item_kinds(&scene).len(), 9);

    scene.add_classes(buttons[0], &["hidden"]);
    scene.frame();

    assert_eq!(item_kinds(&scene).len(), 5);
    assert!(damage(&scene).iter().any(|damaged| damaged.contains(&first_box)));
    assert!(damage(&scene).iter().any(|damaged| damaged.contains(&inside_box)));

    let hidden = scene.class("hidden");
    scene.tree.remove_class(buttons[0], hidden);
    let shown = scene.frame();

    assert_eq!(shown.rebuilt, 4, "the two that came back and the two that moved back");
    assert_eq!(scene.border_box(buttons[1]), second_box);
    assert_eq!(item_kinds(&scene).len(), 9);
    assert_eq!(background_color_of(&scene, inside), scene.light_color("color-surface"));
}

#[test]
fn switching_the_theme_repaints_every_element_in_the_colors_of_the_other_table() {
    let (mut scene, buttons) = toolbar();
    scene.add_classes(buttons[0], &["raised"]);
    scene.frame();

    scene.tree.set_theme(scene.dark_theme());
    let switched = scene.frame();

    assert_eq!(switched.rebuilt, 4);
    assert!(damage(&scene).contains(&Rect::new(0.0, 0.0, 800.0, 600.0)));
    assert_eq!(background_color_of(&scene, scene.root()), scene.dark_color("color-bg"));
    assert_eq!(
        background_color_of(&scene, buttons[1]),
        scene.dark_color("color-surface")
    );
}

#[test]
fn a_gradient_that_did_not_change_keeps_its_stops_when_another_one_changes() {
    let mut scene = Scene::new(
        ".swatch { height: 20px; background-image: linear-gradient(var(--color-accent), var(--color-bg), var(--color-focus)); }\n\
         .swatch:hover { background-image: linear-gradient(var(--color-danger), var(--color-bg)); }",
    );
    let first = scene.add(scene.root(), Role::Element, &["swatch"]);
    scene.add(scene.root(), Role::Element, &["swatch"]);
    scene.frame();
    let before = painted_items(&scene);
    assert_eq!(before[0], before[1].clone_at(&before[0]));

    scene.tree.set_state(first, States::HOVER, true);
    let hovered = scene.frame();

    assert_eq!(hovered.rebuilt, 1);
    let after = painted_items(&scene);
    assert_eq!(after[0].gradient_stops.len(), 2);
    assert_eq!(after[0].gradient_stops[0].color, scene.light_color("color-danger"));
    assert_eq!(after[1], before[1]);
    assert_eq!(scene.tree.display_list().items().len(), 2);
}

impl PaintedItem {
    /// This item as if it belonged to the box of `other`: the two then differ only in what they draw.
    fn clone_at(&self, other: &Self) -> Self {
        let (DisplayItem::Gradient { gradient, .. }, DisplayItem::Gradient { frame, .. }) = (self.item, other.item)
        else {
            panic!("both items are gradients");
        };
        Self {
            item: DisplayItem::Gradient { frame, gradient },
            gradient_stops: self.gradient_stops.clone(),
        }
    }
}

#[test]
#[should_panic(expected = "layout is computed before the display list")]
fn building_the_display_list_of_a_tree_that_is_not_laid_out_panics() {
    let (mut scene, buttons) = toolbar();
    scene.frame();

    scene.add_classes(buttons[0], &["tall"]);
    scene.tree.resolve_styles();
    scene.tree.build_display_list();
}

#[test]
fn a_root_without_a_box_paints_nothing_and_damages_what_the_tree_drew() {
    let (mut scene, _) = toolbar();
    scene.frame();

    scene.add_classes(scene.root(), &["hidden"]);
    let hidden = scene.frame();

    assert_eq!(hidden.rebuilt, 0);
    assert!(scene.tree.display_list().items().is_empty());
    assert_eq!(damage(&scene), [Rect::new(0.0, 0.0, 800.0, 600.0)]);

    let class = scene.class("hidden");
    scene.tree.remove_class(scene.root(), class);
    let shown = scene.frame();

    assert_eq!(shown.rebuilt, 4);
    assert_eq!(item_kinds(&scene).len(), 7);
}
