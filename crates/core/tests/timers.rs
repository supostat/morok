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

/// Remembers every time a tree asked its host for a frame at.
#[derive(Debug, Default)]
struct TimeRecorder {
    requested_times: Rc<RefCell<Vec<Duration>>>,
}

impl FrameScheduler for TimeRecorder {
    fn request_frame(&self) {}

    fn request_frame_at(&self, time: Duration) {
        self.requested_times.borrow_mut().push(time);
    }
}

/// A scene with one element, whose tree has a host, and the times the tree asked the host for
/// frames at.
fn hosted_scene() -> (Scene, ElementId, Rc<RefCell<Vec<Duration>>>) {
    let mut scene = Scene::new(PAGE);
    let note = scene.add(scene.root(), Role::Element, &["note"]);
    let requested_times = Rc::new(RefCell::new(Vec::new()));
    scene.tree.set_frame_scheduler(Box::new(TimeRecorder {
        requested_times: Rc::clone(&requested_times),
    }));
    (scene, note, requested_times)
}

/// A focused field whose caret blinks from the frame at `shown_at` on.
fn focused_field(scene: &mut Scene, shown_at: Duration) -> ElementId {
    let field = scene.add(scene.root(), Role::TextInput, &["field"]);
    scene.frame_at(Duration::ZERO);
    scene.tree.set_focusability(field, Focusability::TabStop);
    assert!(scene.tree.focus(field));
    scene.frame_at(shown_at);
    field
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
    let (mut scene, note, requested_times) = hosted_scene();
    scene.frame_at(milliseconds(1000));

    let timer = scene.tree.start_timer(note, milliseconds(600));

    assert!(
        due_at(&mut scene, milliseconds(5000)).is_empty(),
        "the delay counts from this frame"
    );
    assert_eq!(
        requested_times.borrow().last(),
        Some(&milliseconds(5600)),
        "and the tree asks for the frame at the time it falls due"
    );
    assert!(due_at(&mut scene, milliseconds(5599)).is_empty());
    assert_eq!(
        requested_times.borrow().last(),
        Some(&milliseconds(5600)),
        "every frame before then asks for the same time"
    );
    assert_eq!(due_at(&mut scene, milliseconds(5600)), [timer]);
    assert!(
        due_at(&mut scene, milliseconds(9000)).is_empty(),
        "a timer falls due once"
    );
}

#[test]
fn timers_fall_due_the_earliest_first_and_the_tree_asks_for_the_earliest() {
    let (mut scene, note, requested_times) = hosted_scene();
    let late = scene.tree.start_timer(note, milliseconds(900));
    let early = scene.tree.start_timer(note, milliseconds(200));
    let middle = scene.tree.start_timer(note, milliseconds(500));

    scene.frame_at(milliseconds(100));

    assert_eq!(*requested_times.borrow(), [milliseconds(300)]);
    assert_eq!(due_at(&mut scene, milliseconds(650)), [early, middle]);
    assert_eq!(
        requested_times.borrow().last(),
        Some(&milliseconds(1000)),
        "the tree waits for the one that is left"
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
    let (mut scene, note, requested_times) = hosted_scene();
    focused_field(&mut scene, milliseconds(0));
    let caret_turn = *requested_times
        .borrow()
        .last()
        .expect("a caret that blinks asks for its next turn");

    scene.tree.start_timer(note, caret_turn + milliseconds(300));
    requested_times.borrow_mut().clear();
    scene.frame_at(milliseconds(100));
    assert_eq!(
        *requested_times.borrow(),
        [caret_turn],
        "the caret turns before the timer falls due"
    );

    scene.tree.start_timer(note, milliseconds(40));
    requested_times.borrow_mut().clear();
    scene.frame_at(milliseconds(200));
    assert_eq!(
        *requested_times.borrow(),
        [milliseconds(240)],
        "a timer that falls due before the caret turns is waited for instead, and alone"
    );
}

#[test]
fn every_frame_before_a_turn_of_the_caret_asks_for_the_time_of_that_turn() {
    let (mut scene, _, requested_times) = hosted_scene();
    focused_field(&mut scene, milliseconds(1000));

    for time in [1100, 1200, 1529] {
        scene.frame_at(milliseconds(time));
    }
    assert_eq!(
        *requested_times.borrow(),
        [milliseconds(1530); 4],
        "a caret shown at the frame of 1000 ms turns half a period later"
    );

    requested_times.borrow_mut().clear();
    for time in [1530, 1700, 2059] {
        scene.frame_at(milliseconds(time));
    }
    assert_eq!(
        *requested_times.borrow(),
        [milliseconds(2060); 3],
        "once it has turned, every frame asks for the time of its next turn"
    );
}
