mod support;

use std::cell::Cell;
use std::rc::Rc;

use morok_core::{FrameScheduler, Role};
use morok_style::States;
use support::{InputLog, Scene};

const TOOLBAR: &str = ".app { display: flex; background-color: var(--color-bg); }\n\
    .button { width: 100px; height: 30px; background-color: var(--color-surface); }\n\
    .button:hover { background-color: var(--color-surface-2); }";

/// Counts the frames a tree asked its host for.
#[derive(Debug, Default)]
struct CountingScheduler {
    requested_frames: Rc<Cell<usize>>,
}

impl FrameScheduler for CountingScheduler {
    fn request_frame(&self) {
        self.requested_frames.set(self.requested_frames.get() + 1);
    }
}

/// A change of a scene, made through its tree.
type Change<'a> = &'a dyn Fn(&mut Scene);

/// A scene whose tree has a host, and the number of frames the tree asked the host for.
fn hosted_scene() -> (Scene, Rc<Cell<usize>>) {
    let mut scene = Scene::new(TOOLBAR);
    scene.add_classes(scene.root(), &["app"]);
    let requested_frames = Rc::new(Cell::new(0));
    scene.tree.set_frame_scheduler(Box::new(CountingScheduler {
        requested_frames: Rc::clone(&requested_frames),
    }));
    (scene, requested_frames)
}

#[test]
fn a_tree_that_gets_a_host_asks_for_its_first_frame() {
    let (_scene, requested_frames) = hosted_scene();

    assert_eq!(requested_frames.get(), 1);
}

#[test]
fn a_tree_that_waits_for_nothing_asks_its_new_host_for_nothing() {
    let mut scene = Scene::new(TOOLBAR);
    scene.frame();
    let requested_frames = Rc::new(Cell::new(0));

    scene.tree.set_frame_scheduler(Box::new(CountingScheduler {
        requested_frames: Rc::clone(&requested_frames),
    }));

    assert_eq!(requested_frames.get(), 0);
}

#[test]
fn changes_before_a_frame_ask_for_it_once() {
    let (mut scene, requested_frames) = hosted_scene();
    scene.frame();
    requested_frames.set(0);

    let button = scene.add(scene.root(), Role::Element, &["button"]);
    scene.tree.set_state(button, States::HOVER, true);

    assert_eq!(requested_frames.get(), 1);
}

#[test]
fn a_frame_that_ran_leaves_nothing_to_ask_for() {
    let (mut scene, requested_frames) = hosted_scene();
    scene.add(scene.root(), Role::Element, &["button"]);
    requested_frames.set(0);

    scene.frame();

    assert_eq!(requested_frames.get(), 0);
}

#[test]
fn every_kind_of_change_asks_for_a_frame() {
    let (mut scene, requested_frames) = hosted_scene();
    let button = scene.add(scene.root(), Role::Element, &["button"]);
    let label = scene.add_text(button, "Open");
    let hover = scene.class("button");
    let light_theme = scene.light_theme();
    let changes: [(&str, Change<'_>); 7] = [
        ("a state", &|scene| scene.tree.set_state(button, States::HOVER, true)),
        ("a removed class", &|scene| scene.tree.remove_class(button, hover)),
        ("an added class", &|scene| scene.tree.add_class(button, hover)),
        ("a text", &|scene| scene.tree.set_text(label, "Close")),
        ("a theme", &|scene| scene.tree.set_theme(light_theme.clone())),
        ("a detached element", &|scene| scene.tree.detach(button)),
        ("an inserted element", &|scene| {
            let root = scene.root();
            scene.tree.append_child(root, button);
        }),
    ];

    for (change_name, change) in changes {
        scene.frame();
        requested_frames.set(0);

        change(&mut scene);

        assert_eq!(requested_frames.get(), 1, "{change_name} asks for a frame");
    }
}

#[test]
fn a_change_that_changes_nothing_asks_for_nothing() {
    let (mut scene, requested_frames) = hosted_scene();
    let button = scene.add(scene.root(), Role::Element, &["button"]);
    scene.frame();
    requested_frames.set(0);

    scene.tree.set_state(button, States::HOVER, false);
    scene.tree.add_class(button, scene.class("button"));

    assert_eq!(requested_frames.get(), 0);
}

#[test]
fn a_change_outside_the_tree_asks_for_nothing() {
    let (mut scene, requested_frames) = hosted_scene();
    scene.frame();
    requested_frames.set(0);

    let detached = scene.tree.create(Role::Element);
    scene.tree.add_class(detached, scene.class("button"));
    scene.tree.set_state(detached, States::HOVER, true);

    assert_eq!(requested_frames.get(), 0);
}

#[test]
fn a_change_made_once_a_frame_began_asks_for_the_next_frame() {
    let (mut scene, requested_frames) = hosted_scene();
    let button = scene.add(scene.root(), Role::Element, &["button"]);
    scene.frame();
    scene.tree.set_state(button, States::HOVER, true);
    requested_frames.set(0);

    scene.tree.resolve_styles(scene.time);
    scene.tree.set_state(button, States::HOVER, false);

    assert_eq!(requested_frames.get(), 1);
}

#[test]
fn a_pointer_that_changes_no_state_asks_for_no_frame() {
    let (mut scene, requested_frames) = hosted_scene();
    scene.add(scene.root(), Role::Element, &["button"]);
    scene.frame();
    let mut log = InputLog::default();
    scene.move_pointer(300.0, 300.0, &mut log);
    scene.frame();
    scene.tree.refresh_hover(&mut log);
    scene.frame();
    requested_frames.set(0);

    scene.move_pointer(310.0, 310.0, &mut log);

    assert_eq!(requested_frames.get(), 0);
}

#[test]
fn a_pointer_that_comes_over_an_element_asks_for_the_frame_that_shows_the_hover() {
    let (mut scene, requested_frames) = hosted_scene();
    let button = scene.add(scene.root(), Role::Element, &["button"]);
    scene.frame();
    requested_frames.set(0);
    let mut log = InputLog::default();

    scene.move_pointer(10.0, 10.0, &mut log);

    assert!(scene.tree.states(button).contains(States::HOVER));
    assert_eq!(requested_frames.get(), 1);
}

#[test]
fn a_frame_that_repaints_under_a_resting_pointer_asks_for_one_more_to_follow_the_boxes() {
    let (mut scene, requested_frames) = hosted_scene();
    scene.add(scene.root(), Role::Element, &["button"]);
    scene.frame();
    let mut log = InputLog::default();
    scene.move_pointer(10.0, 10.0, &mut log);
    requested_frames.set(0);

    scene.frame();
    assert_eq!(requested_frames.get(), 1, "the frame painted the hover");

    scene.tree.refresh_hover(&mut log);
    scene.frame();
    assert_eq!(requested_frames.get(), 1, "the boxes rest: nothing more to ask for");
}

#[test]
fn a_frame_that_repaints_with_no_pointer_on_the_surface_asks_for_nothing() {
    let (mut scene, requested_frames) = hosted_scene();
    let button = scene.add(scene.root(), Role::Element, &["button"]);
    scene.frame();
    scene.tree.set_state(button, States::HOVER, true);
    requested_frames.set(0);

    scene.frame();

    assert_eq!(requested_frames.get(), 0);
}
