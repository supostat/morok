mod support;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use morok_core::{ElementId, Focusability, FrameScheduler, Role, TimerId};
use support::Scene;

const PAGE: &str = ".note { width: 100px; height: 30px; }\n\
    .field { width: 200px; color: var(--color-text); }";

const fn milliseconds(count: u64) -> Duration {
    Duration::from_millis(count)
}

/// Remembers every delay a tree asked its host for a frame after.
#[derive(Debug, Default)]
struct DelayRecorder {
    delays: Rc<RefCell<Vec<Duration>>>,
}

impl FrameScheduler for DelayRecorder {
    fn request_frame(&self) {}

    fn request_frame_after(&self, delay: Duration) {
        self.delays.borrow_mut().push(delay);
    }
}

/// A scene with one element, whose tree has a host, and the delays the tree asked the host
/// for frames after.
fn hosted_scene() -> (Scene, ElementId, Rc<RefCell<Vec<Duration>>>) {
    let mut scene = Scene::new(PAGE);
    let note = scene.add(scene.root(), Role::Element, &["note"]);
    let delays = Rc::new(RefCell::new(Vec::new()));
    scene.tree.set_frame_scheduler(Box::new(DelayRecorder {
        delays: Rc::clone(&delays),
    }));
    (scene, note, delays)
}

/// The timers of the scene that fall due by `time`, taken as a host takes them before the
/// stages of the frame at that time.
fn due_at(scene: &mut Scene, time: Duration) -> Vec<TimerId> {
    let mut due = Vec::new();
    scene.tree.take_due_timers(time, &mut due);
    scene.frame_at(time);
    due
}

#[test]
fn a_timer_falls_due_once_its_delay_has_passed_since_the_frame_after_it_was_started() {
    let (mut scene, note, delays) = hosted_scene();
    scene.frame_at(milliseconds(1000));

    let timer = scene.tree.start_timer(note, milliseconds(600));

    assert!(
        due_at(&mut scene, milliseconds(5000)).is_empty(),
        "the delay counts from this frame"
    );
    assert_eq!(
        delays.borrow().last(),
        Some(&milliseconds(600)),
        "and the tree asks for the frame it runs out in"
    );
    assert!(due_at(&mut scene, milliseconds(5599)).is_empty());
    assert_eq!(delays.borrow().last(), Some(&milliseconds(1)));
    assert_eq!(due_at(&mut scene, milliseconds(5600)), [timer]);
    assert!(
        due_at(&mut scene, milliseconds(9000)).is_empty(),
        "a timer falls due once"
    );
}

#[test]
fn timers_fall_due_the_earliest_first_and_the_tree_asks_for_the_earliest() {
    let (mut scene, note, delays) = hosted_scene();
    let late = scene.tree.start_timer(note, milliseconds(900));
    let early = scene.tree.start_timer(note, milliseconds(200));
    let middle = scene.tree.start_timer(note, milliseconds(500));

    scene.frame_at(milliseconds(100));

    assert_eq!(delays.borrow().last(), Some(&milliseconds(200)));
    assert_eq!(due_at(&mut scene, milliseconds(650)), [early, middle]);
    assert_eq!(
        delays.borrow().last(),
        Some(&milliseconds(350)),
        "the tree waits for what is left"
    );
    assert_eq!(due_at(&mut scene, milliseconds(1000)), [late]);
}

#[test]
fn a_cancelled_timer_never_falls_due_and_a_timer_goes_with_its_element() {
    let (mut scene, note, _) = hosted_scene();
    let other = scene.add(scene.root(), Role::Element, &["note"]);
    let cancelled = scene.tree.start_timer(note, milliseconds(100));
    let orphaned = scene.tree.start_timer(other, milliseconds(100));
    let kept = scene.tree.start_timer(note, milliseconds(100));
    scene.frame_at(milliseconds(0));

    scene.tree.cancel_timer(cancelled);
    scene.tree.remove(other);

    assert_eq!(due_at(&mut scene, milliseconds(100)), [kept]);
    assert!(due_at(&mut scene, milliseconds(200)).is_empty());
    scene.tree.cancel_timer(orphaned);
}

#[test]
fn of_a_caret_and_a_timer_the_tree_asks_for_the_frame_of_whichever_turns_first() {
    let (mut scene, note, delays) = hosted_scene();
    let field = scene.add(scene.root(), Role::TextInput, &["field"]);
    scene.frame_at(milliseconds(0));
    scene.tree.set_focusability(field, Focusability::TabStop);
    assert!(scene.tree.focus(field));
    scene.frame_at(milliseconds(0));
    let caret_turn = *delays
        .borrow()
        .last()
        .expect("a caret that blinks asks for its next turn");

    scene.tree.start_timer(note, caret_turn + milliseconds(300));
    delays.borrow_mut().clear();
    scene.frame_at(milliseconds(100));
    assert_eq!(
        *delays.borrow(),
        [caret_turn.saturating_sub(milliseconds(100))],
        "the caret turns before the timer falls due"
    );

    scene.tree.start_timer(note, milliseconds(40));
    delays.borrow_mut().clear();
    scene.frame_at(milliseconds(200));
    assert_eq!(
        delays.borrow().last(),
        Some(&milliseconds(40)),
        "a timer that falls due before the caret turns is waited for instead"
    );
}
