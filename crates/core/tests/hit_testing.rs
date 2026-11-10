mod support;

use morok_core::{ElementId, Point, PointerButton, PointerEventKind, PointerInputKind, Rect, Role};
use morok_style::States;
use support::{InputLog, Scene};

const STAGE: &str = ".stage { position: relative; width: 400px; height: 300px; }\n\
    .card { position: absolute; left: 100px; top: 100px; width: 200px; height: 100px; \
            background-color: var(--color-surface); }\n\
    .glass { position: absolute; left: 150px; top: 120px; width: 200px; height: 100px; }\n\
    .pill { position: absolute; left: 100px; top: 100px; width: 200px; height: 100px; \
            border-radius: var(--radius-full); background-color: var(--color-surface); }\n\
    .viewport { overflow: hidden; width: 100px; height: 50px; }\n\
    .round { border-radius: var(--radius-xl); }\n\
    .tall { height: 200px; background-color: var(--color-surface); }\n\
    .anchor { position: relative; width: 200px; height: 100px; }\n\
    .scroller { overflow: hidden; height: 50px; }\n\
    .popup { position: absolute; left: 0; top: 60px; width: 50px; height: 30px; }\n\
    .moved { transform: translateX(150px); }\n\
    .turned { transform: rotate(45deg); }\n\
    .grown { transform: scale(2); }\n\
    .flat { transform: scaleX(0); }\n\
    .square { position: absolute; left: 100px; top: 100px; width: 100px; height: 100px; }\n\
    .dot { width: 10px; height: 10px; }";

const TOOLBAR: &str = ".toolbar { display: flex; gap: 20px; padding: 20px; width: 400px; }\n\
    .button { display: flex; width: 100px; height: 40px; padding: 10px; background-color: var(--color-surface); }\n\
    .button:hover { background-color: var(--color-surface-2); }\n\
    .button:active { background-color: var(--color-accent); }\n\
    .gone { display: none; }";

fn point(x: f32, y: f32) -> Point {
    Point { x, y }
}

/// A scene whose root is a stage of 400 by 300 pixels, with one element of each class on it.
fn stage_with(classes: &[&str]) -> (Scene, Vec<ElementId>) {
    let mut scene = Scene::new(STAGE);
    scene.add_classes(scene.root(), &["stage"]);
    let elements = classes
        .iter()
        .map(|class| scene.add(scene.root(), Role::Element, &[class]))
        .collect();
    scene.frame();
    (scene, elements)
}

/// A toolbar of two buttons, the first with a text in it, after its first frame.
struct ToolbarScene {
    scene: Scene,
    toolbar: ElementId,
    save: ElementId,
    save_label: ElementId,
    cancel: ElementId,
}

impl ToolbarScene {
    fn new() -> Self {
        let mut scene = Scene::new(TOOLBAR);
        let toolbar = scene.add(scene.root(), Role::Element, &["toolbar"]);
        let save = scene.add(toolbar, Role::Element, &["button"]);
        let save_label = scene.add_text(save, "Save");
        let cancel = scene.add(toolbar, Role::Element, &["button"]);
        scene.frame();
        Self {
            scene,
            toolbar,
            save,
            save_label,
            cancel,
        }
    }

    fn states(&self, element: ElementId) -> States {
        self.scene.tree.states(element)
    }

    fn is_hovered(&self, element: ElementId) -> bool {
        self.states(element).contains(States::HOVER)
    }

    fn is_active(&self, element: ElementId) -> bool {
        self.states(element).contains(States::ACTIVE)
    }
}

/// A point over the text of the first button, one over its padding and one over the second button.
const OVER_SAVE_LABEL: (f32, f32) = (35.0, 35.0);
const OVER_SAVE_PADDING: (f32, f32) = (115.0, 55.0);
const OVER_CANCEL: (f32, f32) = (200.0, 40.0);
const OVER_TOOLBAR_PADDING: (f32, f32) = (5.0, 5.0);

#[test]
fn the_box_painted_last_under_the_point_is_hit_whatever_it_paints() {
    let (mut scene, elements) = stage_with(&["card", "glass"]);
    let (card, glass) = (elements[0], elements[1]);

    assert_eq!(scene.tree.element_at(point(120.0, 110.0)), Some(card));
    assert_eq!(scene.tree.element_at(point(200.0, 150.0)), Some(glass));
    assert_eq!(scene.tree.element_at(point(340.0, 210.0)), Some(glass));
    assert_eq!(scene.tree.element_at(point(50.0, 50.0)), Some(scene.root()));
    assert_eq!(scene.tree.element_at(point(500.0, 400.0)), None);
}

#[test]
fn a_point_in_the_cut_of_a_rounded_corner_hits_what_lies_under_the_box() {
    let (mut scene, elements) = stage_with(&["pill"]);
    let pill = elements[0];

    assert_eq!(scene.tree.element_at(point(104.0, 104.0)), Some(scene.root()));
    assert_eq!(scene.tree.element_at(point(296.0, 196.0)), Some(scene.root()));
    assert_eq!(scene.tree.element_at(point(120.0, 120.0)), Some(pill));
    assert_eq!(scene.tree.element_at(point(102.0, 150.0)), Some(pill));
}

#[test]
fn content_that_a_clip_hides_is_not_hit() {
    let (mut scene, elements) = stage_with(&["viewport"]);
    let viewport = elements[0];
    let tall = scene.add(viewport, Role::Element, &["tall"]);
    scene.frame();

    assert_eq!(scene.tree.element_at(point(50.0, 25.0)), Some(tall));
    assert!(
        scene.border_box(tall).bottom() > 100.0,
        "the box of the content reaches below the clip"
    );
    assert_eq!(scene.tree.element_at(point(50.0, 100.0)), Some(scene.root()));
}

#[test]
fn a_rounded_clip_hides_the_content_in_its_corners() {
    let (mut scene, elements) = stage_with(&["viewport"]);
    let viewport = elements[0];
    scene.add_classes(viewport, &["round"]);
    let tall = scene.add(viewport, Role::Element, &["tall"]);
    scene.frame();

    assert_eq!(scene.tree.element_at(point(2.0, 2.0)), Some(scene.root()));
    assert_eq!(scene.tree.element_at(point(20.0, 20.0)), Some(tall));
}

#[test]
fn an_absolute_element_is_hit_outside_a_clip_that_does_not_contain_it() {
    let (mut scene, elements) = stage_with(&["anchor"]);
    let anchor = elements[0];
    let scroller = scene.add(anchor, Role::Element, &["scroller"]);
    let popup = scene.add(scroller, Role::Element, &["popup"]);
    scene.frame();

    assert_eq!(scene.tree.element_at(point(25.0, 75.0)), Some(popup));
    assert_eq!(scene.tree.element_at(point(100.0, 75.0)), Some(anchor));
}

#[test]
fn a_moved_box_is_hit_where_it_is_shown_and_not_where_it_was_laid_out() {
    let (mut scene, elements) = stage_with(&["square"]);
    let square = elements[0];
    scene.add_classes(square, &["moved"]);
    let dot = scene.add(square, Role::Element, &["dot"]);
    scene.frame();

    assert_eq!(scene.tree.element_at(point(150.0, 150.0)), Some(scene.root()));
    assert_eq!(scene.tree.element_at(point(300.0, 150.0)), Some(square));
    assert_eq!(scene.tree.element_at(point(255.0, 105.0)), Some(dot));
}

#[test]
fn every_element_that_hears_of_the_pointer_is_told_its_box_as_the_surface_shows_it() {
    let (mut scene, elements) = stage_with(&["square"]);
    let square = elements[0];
    scene.add_classes(square, &["moved"]);
    let dot = scene.add(square, Role::Element, &["dot"]);
    scene.frame();
    let mut log = InputLog::default();

    scene.press(255.0, 105.0, &mut log);

    let pressed = PointerEventKind::Pressed {
        button: PointerButton::Primary,
        click_count: 1,
    };
    let told: Vec<_> = log
        .pointer_events
        .iter()
        .zip(&log.pointer_event_boxes)
        .filter(|((_, kind), _)| *kind == pressed)
        .map(|((element, _), element_box)| (*element, *element_box))
        .collect();
    assert_eq!(
        told,
        [
            (dot, Rect::new(250.0, 100.0, 10.0, 10.0)),
            (square, Rect::new(250.0, 100.0, 100.0, 100.0)),
            (scene.root(), Rect::new(0.0, 0.0, 400.0, 300.0)),
        ],
        "the press goes from the innermost element out, each told the box moved by its transforms"
    );
}

#[test]
fn a_turned_box_is_hit_by_its_turned_outline() {
    let (mut scene, elements) = stage_with(&["square"]);
    let square = elements[0];
    scene.add_classes(square, &["turned"]);
    scene.frame();

    assert_eq!(scene.tree.element_at(point(104.0, 104.0)), Some(scene.root()));
    assert_eq!(scene.tree.element_at(point(150.0, 85.0)), Some(square));
    assert_eq!(scene.tree.element_at(point(150.0, 150.0)), Some(square));
}

#[test]
fn a_grown_box_is_hit_over_all_it_covers_and_a_flattened_one_nowhere() {
    let (mut scene, elements) = stage_with(&["square"]);
    let square = elements[0];
    scene.add_classes(square, &["grown"]);
    scene.frame();
    assert_eq!(scene.tree.element_at(point(60.0, 60.0)), Some(square));
    assert_eq!(scene.tree.element_at(point(240.0, 240.0)), Some(square));

    let grown = scene.class("grown");
    scene.tree.remove_class(square, grown);
    scene.add_classes(square, &["flat"]);
    scene.frame();
    assert_eq!(scene.tree.element_at(point(150.0, 150.0)), Some(scene.root()));
}

#[test]
fn an_element_taken_out_since_the_last_frame_is_not_hit() {
    let (mut scene, elements) = stage_with(&["card", "glass"]);
    let (card, glass) = (elements[0], elements[1]);

    scene.tree.remove(glass);
    scene.tree.detach(card);

    assert_eq!(scene.tree.element_at(point(200.0, 150.0)), Some(scene.root()));
}

#[test]
fn hover_follows_the_pointer_through_an_element_and_its_ancestors() {
    let mut toolbar = ToolbarScene::new();
    let mut log = InputLog::default();
    let root = toolbar.scene.root();

    toolbar
        .scene
        .move_pointer(OVER_SAVE_LABEL.0, OVER_SAVE_LABEL.1, &mut log);
    assert!(toolbar.is_hovered(toolbar.save_label) && toolbar.is_hovered(toolbar.save));
    assert!(toolbar.is_hovered(toolbar.toolbar) && toolbar.is_hovered(root));
    assert!(!toolbar.is_hovered(toolbar.cancel));
    assert_eq!(
        log.receivers_of(PointerEventKind::Entered),
        [root, toolbar.toolbar, toolbar.save, toolbar.save_label]
    );

    let mut log = InputLog::default();
    toolbar.scene.move_pointer(OVER_CANCEL.0, OVER_CANCEL.1, &mut log);
    assert!(!toolbar.is_hovered(toolbar.save_label) && !toolbar.is_hovered(toolbar.save));
    assert!(toolbar.is_hovered(toolbar.cancel) && toolbar.is_hovered(toolbar.toolbar));
    assert_eq!(
        log.receivers_of(PointerEventKind::Left),
        [toolbar.save_label, toolbar.save]
    );
    assert_eq!(log.receivers_of(PointerEventKind::Entered), [toolbar.cancel]);
    assert_eq!(
        log.receivers_of(PointerEventKind::Moved),
        [toolbar.cancel, toolbar.toolbar, root]
    );
}

#[test]
fn a_hovered_element_is_restyled_by_its_hover_rule() {
    let mut toolbar = ToolbarScene::new();
    let mut log = InputLog::default();
    let resting = toolbar.scene.tree.style(toolbar.cancel).paint.background_color;

    toolbar.scene.move_pointer(OVER_CANCEL.0, OVER_CANCEL.1, &mut log);
    toolbar.scene.frame();

    assert_ne!(toolbar.scene.tree.style(toolbar.cancel).paint.background_color, resting);
}

#[test]
fn the_pointer_leaving_the_surface_ends_hover() {
    let mut toolbar = ToolbarScene::new();
    let mut log = InputLog::default();
    let root = toolbar.scene.root();
    toolbar.scene.move_pointer(OVER_CANCEL.0, OVER_CANCEL.1, &mut log);

    toolbar.scene.pointer(PointerInputKind::Exited, 0.0, 0.0, &mut log);

    assert!(!toolbar.is_hovered(toolbar.cancel) && !toolbar.is_hovered(root));
    assert_eq!(
        log.receivers_of(PointerEventKind::Left),
        [toolbar.cancel, toolbar.toolbar, root]
    );
}

#[test]
fn a_press_makes_the_element_and_its_ancestors_active_until_the_release() {
    let mut toolbar = ToolbarScene::new();
    let mut log = InputLog::default();
    let root = toolbar.scene.root();

    toolbar.scene.press(OVER_CANCEL.0, OVER_CANCEL.1, &mut log);
    assert!(toolbar.is_active(toolbar.cancel) && toolbar.is_active(toolbar.toolbar) && toolbar.is_active(root));
    assert!(toolbar.is_hovered(toolbar.cancel));
    assert!(!toolbar.is_active(toolbar.save));

    toolbar.scene.release(OVER_CANCEL.0, OVER_CANCEL.1, &mut log);
    assert!(!toolbar.is_active(toolbar.cancel) && !toolbar.is_active(toolbar.toolbar) && !toolbar.is_active(root));
    assert!(toolbar.is_hovered(toolbar.cancel));
    let pressed = PointerEventKind::Pressed {
        button: PointerButton::Primary,
        click_count: 1,
    };
    assert_eq!(log.receivers_of(pressed), [toolbar.cancel, toolbar.toolbar, root]);
    assert_eq!(
        log.receivers_of(PointerEventKind::Clicked { click_count: 1 }),
        [toolbar.cancel, toolbar.toolbar, root]
    );
}

#[test]
fn a_pressed_element_keeps_the_pointer_until_the_release() {
    let mut toolbar = ToolbarScene::new();
    let mut log = InputLog::default();
    let root = toolbar.scene.root();
    toolbar.scene.press(OVER_SAVE_PADDING.0, OVER_SAVE_PADDING.1, &mut log);

    let mut log = InputLog::default();
    toolbar.scene.move_pointer(OVER_CANCEL.0, OVER_CANCEL.1, &mut log);
    assert!(toolbar.is_active(toolbar.save), "the pressed element stays active");
    assert!(!toolbar.is_hovered(toolbar.save), "the pointer is no longer over it");
    assert!(!toolbar.is_hovered(toolbar.cancel), "nothing outside it is hovered");
    assert!(toolbar.is_hovered(toolbar.toolbar));
    assert_eq!(
        log.receivers_of(PointerEventKind::Moved),
        [toolbar.save, toolbar.toolbar, root]
    );
    assert_eq!(log.receivers_of(PointerEventKind::Entered), []);

    let mut log = InputLog::default();
    toolbar.scene.release(OVER_CANCEL.0, OVER_CANCEL.1, &mut log);
    let released = PointerEventKind::Released {
        button: PointerButton::Primary,
    };
    assert_eq!(log.receivers_of(released), [toolbar.save, toolbar.toolbar, root]);
    assert_eq!(
        log.receivers_of(PointerEventKind::Clicked { click_count: 1 }),
        [toolbar.toolbar, root],
        "the buttons were not clicked: the release was over another one"
    );
    assert!(!toolbar.is_active(toolbar.save));
    assert!(
        toolbar.is_hovered(toolbar.cancel),
        "the released pointer hovers what it is over"
    );
    assert_eq!(log.receivers_of(PointerEventKind::Entered), [toolbar.cancel]);
}

#[test]
fn a_pressed_element_is_hovered_again_when_the_pointer_comes_back() {
    let mut toolbar = ToolbarScene::new();
    let mut log = InputLog::default();
    toolbar.scene.press(OVER_SAVE_PADDING.0, OVER_SAVE_PADDING.1, &mut log);
    toolbar.scene.move_pointer(OVER_CANCEL.0, OVER_CANCEL.1, &mut log);

    toolbar
        .scene
        .move_pointer(OVER_SAVE_PADDING.0, OVER_SAVE_PADDING.1, &mut log);

    assert!(toolbar.is_hovered(toolbar.save) && toolbar.is_active(toolbar.save));
}

#[test]
fn a_click_belongs_to_the_innermost_element_around_the_press_and_the_release() {
    let mut toolbar = ToolbarScene::new();
    let mut log = InputLog::default();
    log.consumers.push(toolbar.save);

    toolbar.scene.press(OVER_SAVE_LABEL.0, OVER_SAVE_LABEL.1, &mut log);
    toolbar
        .scene
        .release(OVER_SAVE_PADDING.0, OVER_SAVE_PADDING.1, &mut log);

    assert_eq!(
        log.receivers_of(PointerEventKind::Clicked { click_count: 1 }),
        [toolbar.save],
        "the button used the click, so its ancestors did not hear of it"
    );
    let pressed = PointerEventKind::Pressed {
        button: PointerButton::Primary,
        click_count: 1,
    };
    assert_eq!(log.receivers_of(pressed), [toolbar.save_label, toolbar.save]);
}

#[test]
fn a_secondary_press_is_told_and_holds_nothing() {
    let mut toolbar = ToolbarScene::new();
    let mut log = InputLog::default();
    let pressed = PointerInputKind::Pressed {
        button: PointerButton::Secondary,
        click_count: 1,
    };

    toolbar.scene.pointer(pressed, OVER_CANCEL.0, OVER_CANCEL.1, &mut log);
    toolbar
        .scene
        .move_pointer(OVER_SAVE_PADDING.0, OVER_SAVE_PADDING.1, &mut log);

    assert!(!toolbar.is_active(toolbar.cancel));
    assert!(toolbar.is_hovered(toolbar.save));
    let told = PointerEventKind::Pressed {
        button: PointerButton::Secondary,
        click_count: 1,
    };
    assert_eq!(log.receivers_of(told).first(), Some(&toolbar.cancel));
}

#[test]
fn hover_follows_the_boxes_that_move_under_a_resting_pointer() {
    let mut toolbar = ToolbarScene::new();
    let mut log = InputLog::default();
    toolbar
        .scene
        .move_pointer(OVER_SAVE_PADDING.0, OVER_SAVE_PADDING.1, &mut log);
    toolbar.scene.frame();

    toolbar.scene.add_classes(toolbar.save, &["gone"]);
    toolbar.scene.frame();
    let mut log = InputLog::default();
    toolbar.scene.tree.refresh_hover(&mut log);

    assert!(!toolbar.is_hovered(toolbar.save));
    assert!(
        toolbar.is_hovered(toolbar.cancel),
        "the second button moved under the pointer"
    );
    assert_eq!(log.receivers_of(PointerEventKind::Left), [toolbar.save]);
    assert_eq!(log.receivers_of(PointerEventKind::Entered), [toolbar.cancel]);
    assert_eq!(
        log.receivers_of(PointerEventKind::Moved),
        [],
        "the pointer did not move"
    );
}

#[test]
fn a_frame_that_moves_nothing_leaves_hover_alone() {
    let mut toolbar = ToolbarScene::new();
    let mut log = InputLog::default();
    toolbar
        .scene
        .move_pointer(OVER_TOOLBAR_PADDING.0, OVER_TOOLBAR_PADDING.1, &mut log);
    toolbar.scene.frame();
    toolbar.scene.frame();

    let mut log = InputLog::default();
    toolbar.scene.tree.refresh_hover(&mut log);

    assert_eq!(log.pointer_events, []);
    assert!(toolbar.is_hovered(toolbar.toolbar));
}

#[test]
fn an_element_taken_out_of_the_tree_lets_go_of_the_pointer() {
    let mut toolbar = ToolbarScene::new();
    let mut log = InputLog::default();
    let root = toolbar.scene.root();
    toolbar.scene.press(OVER_SAVE_LABEL.0, OVER_SAVE_LABEL.1, &mut log);

    toolbar.scene.tree.remove(toolbar.save);

    assert!(!toolbar.is_active(toolbar.toolbar) && !toolbar.is_active(root));
    assert!(
        toolbar.is_hovered(toolbar.toolbar),
        "hover falls back on what was around it"
    );
    let mut log = InputLog::default();
    toolbar.scene.frame();
    toolbar.scene.tree.refresh_hover(&mut log);
    toolbar.scene.release(OVER_SAVE_LABEL.0, OVER_SAVE_LABEL.1, &mut log);
    assert_eq!(log.receivers_of(PointerEventKind::Clicked { click_count: 1 }), []);
    assert!(
        toolbar.is_hovered(toolbar.cancel),
        "the second button took the place of the first"
    );
}
