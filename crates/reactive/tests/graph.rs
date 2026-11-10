use std::cell::{Cell, RefCell};
use std::rc::Rc;

use morok_reactive::{Memo, Runtime};

fn counter() -> Rc<Cell<u32>> {
    Rc::new(Cell::new(0))
}

fn count(counter: &Rc<Cell<u32>>) {
    counter.set(counter.get() + 1);
}

fn flush(runtime: &mut Runtime) {
    runtime.flush(&mut ()).expect("the graph has no cycle");
}

#[test]
fn an_effect_runs_in_the_flush_after_its_creation_and_not_before() {
    let mut runtime: Runtime = Runtime::new();
    let runs = counter();
    let effect_runs = Rc::clone(&runs);
    runtime.effect(move |_, ()| count(&effect_runs));
    assert_eq!(runs.get(), 0);
    flush(&mut runtime);
    flush(&mut runtime);
    assert_eq!(runs.get(), 1);
}

#[test]
fn a_memo_is_computed_on_its_first_read_and_again_only_after_a_source_changed() {
    let mut runtime: Runtime = Runtime::new();
    let computations = counter();
    let width = runtime.signal(4);
    let memo_computations = Rc::clone(&computations);
    let area = runtime.memo(move |runtime| {
        count(&memo_computations);
        runtime.get(width) * 10
    });
    assert_eq!(computations.get(), 0);
    assert_eq!(runtime.get(area), 40);
    assert_eq!(runtime.get(area), 40);
    assert_eq!(computations.get(), 1);
    runtime.set(width, 5);
    assert_eq!(runtime.get(area), 50);
    assert_eq!(computations.get(), 2);
}

#[test]
fn a_value_is_read_in_place_and_updated_in_place() {
    let mut runtime: Runtime = Runtime::new();
    let names = runtime.signal(vec!["a".to_owned()]);
    runtime.update(names, |names| names.push("b".to_owned()));
    assert_eq!(runtime.with(names, Vec::len), 2);
}

#[test]
fn a_memo_that_computes_an_equal_value_wakes_no_effect() {
    let mut runtime: Runtime = Runtime::new();
    let runs = counter();
    let number = runtime.signal(1);
    let is_positive = runtime.memo(move |runtime| runtime.get(number) > 0);
    let effect_runs = Rc::clone(&runs);
    runtime.effect(move |runtime, ()| {
        runtime.get(is_positive);
        count(&effect_runs);
    });
    flush(&mut runtime);
    runtime.set(number, 2);
    flush(&mut runtime);
    assert_eq!(runs.get(), 1);
    runtime.set(number, -1);
    flush(&mut runtime);
    assert_eq!(runs.get(), 2);
}

#[test]
fn an_effect_depends_only_on_what_its_last_run_read() {
    let mut runtime: Runtime = Runtime::new();
    let runs = counter();
    let shows_first = runtime.signal(true);
    let first = runtime.signal(1);
    let second = runtime.signal(2);
    let effect_runs = Rc::clone(&runs);
    runtime.effect(move |runtime, ()| {
        count(&effect_runs);
        if runtime.get(shows_first) {
            runtime.get(first);
        } else {
            runtime.get(second);
        }
    });
    flush(&mut runtime);
    runtime.set(second, 20);
    flush(&mut runtime);
    assert_eq!(runs.get(), 1);

    runtime.set(shows_first, false);
    flush(&mut runtime);
    assert_eq!(runs.get(), 2);
    runtime.set(first, 10);
    flush(&mut runtime);
    assert_eq!(runs.get(), 2);
    runtime.set(second, 30);
    flush(&mut runtime);
    assert_eq!(runs.get(), 3);
}

#[test]
fn in_a_diamond_the_effect_runs_once_and_sees_values_that_agree() {
    let mut runtime: Runtime = Runtime::new();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let base = runtime.signal(1);
    let next = runtime.memo(move |runtime| runtime.get(base) + 1);
    let doubled = runtime.memo(move |runtime| runtime.get(base) * 2);
    let effect_seen = Rc::clone(&seen);
    runtime.effect(move |runtime, ()| {
        let pair = (runtime.get(next), runtime.get(doubled));
        effect_seen.borrow_mut().push(pair);
    });
    flush(&mut runtime);
    runtime.set(base, 10);
    flush(&mut runtime);
    assert_eq!(*seen.borrow(), [(2, 2), (11, 20)]);
}

#[test]
fn writes_before_a_flush_run_an_effect_once() {
    let mut runtime: Runtime = Runtime::new();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let first = runtime.signal(1);
    let second = runtime.signal(2);
    let effect_seen = Rc::clone(&seen);
    runtime.effect(move |runtime, ()| {
        let sum = runtime.get(first) + runtime.get(second);
        effect_seen.borrow_mut().push(sum);
    });
    flush(&mut runtime);
    runtime.set(first, 10);
    runtime.set(second, 20);
    runtime.set(first, 100);
    flush(&mut runtime);
    assert_eq!(*seen.borrow(), [3, 120]);
}

#[test]
fn an_untracked_read_adds_no_dependency() {
    let mut runtime: Runtime = Runtime::new();
    let runs = counter();
    let tracked = runtime.signal(1);
    let untracked = runtime.signal(2);
    let effect_runs = Rc::clone(&runs);
    runtime.effect(move |runtime, ()| {
        count(&effect_runs);
        runtime.get(tracked);
        runtime.untracked(|runtime| runtime.get(untracked));
    });
    flush(&mut runtime);
    runtime.set(untracked, 20);
    flush(&mut runtime);
    assert_eq!(runs.get(), 1);
    runtime.set(tracked, 10);
    flush(&mut runtime);
    assert_eq!(runs.get(), 2);
}

#[test]
fn a_disposed_scope_takes_its_effects_and_nested_scopes_with_it() {
    let mut runtime: Runtime = Runtime::new();
    let outer_runs = counter();
    let inner_runs = counter();
    let number = runtime.signal(1);
    let (outer_effect_runs, inner_effect_runs) = (Rc::clone(&outer_runs), Rc::clone(&inner_runs));
    let scope = runtime.scope(move |runtime| {
        runtime.effect(move |runtime, ()| {
            runtime.get(number);
            count(&outer_effect_runs);
        });
        runtime.scope(move |runtime| {
            runtime.effect(move |runtime, ()| {
                runtime.get(number);
                count(&inner_effect_runs);
            });
        });
    });
    flush(&mut runtime);
    runtime.set(number, 2);
    runtime.dispose(scope);
    flush(&mut runtime);
    runtime.set(number, 3);
    flush(&mut runtime);
    assert_eq!((outer_runs.get(), inner_runs.get()), (1, 1));
}

#[test]
fn disposing_a_scope_leaves_its_siblings_running() {
    let mut runtime: Runtime = Runtime::new();
    let runs = counter();
    let number = runtime.signal(1);
    let mut scopes = Vec::new();
    for _ in 0..3 {
        let effect_runs = Rc::clone(&runs);
        scopes.push(runtime.scope(move |runtime| {
            runtime.effect(move |runtime, ()| {
                runtime.get(number);
                count(&effect_runs);
            });
        }));
    }
    flush(&mut runtime);
    runtime.dispose(scopes[1]);
    runtime.set(number, 2);
    flush(&mut runtime);
    assert_eq!(runs.get(), 5);
    runtime.dispose(scopes[0]);
    runtime.dispose(scopes[2]);
    runtime.set(number, 3);
    flush(&mut runtime);
    assert_eq!(runs.get(), 5);
}

#[test]
fn an_effect_that_runs_again_disposes_what_its_last_run_created() {
    let mut runtime: Runtime = Runtime::new();
    let inner_runs = counter();
    let rebuilds = runtime.signal(0);
    let number = runtime.signal(1);
    let inner_effect_runs = Rc::clone(&inner_runs);
    runtime.effect(move |runtime, ()| {
        runtime.get(rebuilds);
        let inner_effect_runs = Rc::clone(&inner_effect_runs);
        runtime.effect(move |runtime, ()| {
            runtime.get(number);
            count(&inner_effect_runs);
        });
    });
    flush(&mut runtime);
    assert_eq!(inner_runs.get(), 1);
    runtime.set(rebuilds, 1);
    flush(&mut runtime);
    assert_eq!(inner_runs.get(), 2);
    runtime.set(number, 2);
    flush(&mut runtime);
    assert_eq!(inner_runs.get(), 3);
}

#[test]
fn a_stale_owner_runs_first_and_its_old_effects_do_not_run() {
    let mut runtime: Runtime = Runtime::new();
    let inner_runs = counter();
    let number = runtime.signal(1);
    let inner_effect_runs = Rc::clone(&inner_runs);
    let number_through_a_memo = runtime.memo(move |runtime| runtime.get(number));
    // The inner effect reads the signal itself and its owner reads it through a memo, so a
    // write queues the inner effect first.
    runtime.effect(move |runtime, ()| {
        runtime.get(number_through_a_memo);
        let inner_effect_runs = Rc::clone(&inner_effect_runs);
        runtime.effect(move |runtime, ()| {
            runtime.get(number);
            count(&inner_effect_runs);
        });
    });
    flush(&mut runtime);
    assert_eq!(inner_runs.get(), 1);
    runtime.set(number, 2);
    flush(&mut runtime);
    assert_eq!(inner_runs.get(), 2);
}

#[test]
fn an_effect_that_writes_a_signal_wakes_its_readers_in_the_same_flush() {
    let mut runtime: Runtime = Runtime::new();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let celsius = runtime.signal(0);
    let fahrenheit = runtime.signal(32);
    let effect_seen = Rc::clone(&seen);
    runtime.effect(move |runtime, ()| effect_seen.borrow_mut().push(runtime.get(fahrenheit)));
    runtime.effect(move |runtime, ()| {
        let converted = runtime.get(celsius) * 9 / 5 + 32;
        runtime.set(fahrenheit, converted);
    });
    flush(&mut runtime);
    runtime.set(celsius, 100);
    flush(&mut runtime);
    assert_eq!(seen.borrow().last(), Some(&212));
    assert_eq!(runtime.get(fahrenheit), 212);
}

#[test]
fn an_effect_created_by_an_effect_runs_in_the_same_flush() {
    let mut runtime: Runtime = Runtime::new();
    let inner_runs = counter();
    let inner_effect_runs = Rc::clone(&inner_runs);
    runtime.effect(move |runtime, ()| {
        let inner_effect_runs = Rc::clone(&inner_effect_runs);
        runtime.effect(move |_, ()| count(&inner_effect_runs));
    });
    flush(&mut runtime);
    assert_eq!(inner_runs.get(), 1);
}

#[test]
fn an_effect_that_settles_after_writing_what_it_reads_is_not_a_cycle() {
    let mut runtime: Runtime = Runtime::new();
    let level = runtime.signal(250);
    runtime.effect(move |runtime, ()| {
        if runtime.get(level) > 100 {
            runtime.set(level, 100);
        }
    });
    flush(&mut runtime);
    assert_eq!(runtime.get(level), 100);
}

#[test]
fn an_effect_that_keeps_waking_itself_is_reported_as_a_cycle() {
    let mut runtime: Runtime = Runtime::new();
    let runs = counter();
    let number = runtime.signal(0);
    let effect_runs = Rc::clone(&runs);
    let effect = runtime.effect(move |runtime, ()| {
        count(&effect_runs);
        let next = runtime.get(number) + 1;
        runtime.set(number, next);
    });
    let cycle = runtime.flush(&mut ()).expect_err("the effect never settles");
    assert_eq!(cycle.effect, effect);
    assert_eq!(runs.get(), 100);

    let runs_after_the_report = runs.get();
    flush(&mut runtime);
    assert_eq!(runs.get(), runs_after_the_report);
}

#[test]
fn two_effects_that_keep_waking_each_other_are_reported_as_a_cycle() {
    let mut runtime: Runtime = Runtime::new();
    let request = runtime.signal(0);
    let reply = runtime.signal(0);
    runtime.effect(move |runtime, ()| {
        let next = runtime.get(request) + 1;
        runtime.set(reply, next);
    });
    runtime.effect(move |runtime, ()| {
        let next = runtime.get(reply) + 1;
        runtime.set(request, next);
    });
    assert!(runtime.flush(&mut ()).is_err());
}

#[test]
#[should_panic(expected = "a memo does not read its own value while computing it")]
fn a_memo_that_reads_itself_is_reported() {
    let mut runtime: Runtime = Runtime::new();
    let itself: Rc<Cell<Option<Memo<i32>>>> = Rc::new(Cell::new(None));
    let memo_itself = Rc::clone(&itself);
    let memo = runtime.memo(move |runtime| {
        let itself = memo_itself.get().expect("the handle is stored before the first read");
        runtime.get(itself) + 1
    });
    itself.set(Some(memo));
    runtime.get(memo);
}

#[test]
#[should_panic(expected = "a handle is not used after the scope that owns it is disposed")]
fn a_handle_of_a_disposed_scope_is_refused() {
    let mut runtime: Runtime = Runtime::new();
    let mut signal = None;
    let scope = runtime.scope(|runtime| signal = Some(runtime.signal(1)));
    runtime.dispose(scope);
    runtime.get(signal.expect("the scope created the signal"));
}

#[test]
fn an_effect_acts_on_the_world_of_the_flush_that_runs_it() {
    let mut runtime: Runtime<Vec<i32>> = Runtime::new();
    let mut seen = Vec::new();
    let reading = runtime.signal(7);
    runtime.effect(move |runtime, seen: &mut Vec<i32>| seen.push(runtime.get(reading)));
    runtime.flush(&mut seen).expect("the graph has no cycle");
    runtime.set(reading, 9);
    runtime.flush(&mut seen).expect("the graph has no cycle");
    assert_eq!(seen, [7, 9]);
}

#[test]
fn a_flush_has_work_from_the_creation_of_an_effect_or_a_write_it_reads_until_it_ran() {
    let mut runtime: Runtime = Runtime::new();
    let number = runtime.signal(1);
    assert!(!runtime.has_queued_effects());
    runtime.effect(move |runtime, ()| {
        runtime.get(number);
    });
    assert!(runtime.has_queued_effects());
    flush(&mut runtime);
    assert!(!runtime.has_queued_effects());
    runtime.set(number, 2);
    assert!(runtime.has_queued_effects());
    flush(&mut runtime);
    assert!(!runtime.has_queued_effects());
}

#[test]
fn a_scope_created_in_another_scope_outlives_the_run_that_created_it() {
    let mut runtime: Runtime = Runtime::new();
    let kept_runs = counter();
    let owned_runs = counter();
    let trigger = runtime.signal(0);
    let tick = runtime.signal(0);
    let keeper = runtime.scope(|_| {});
    let made = Rc::new(Cell::new(false));
    let (effect_kept_runs, effect_owned_runs) = (Rc::clone(&kept_runs), Rc::clone(&owned_runs));
    runtime.effect(move |runtime, ()| {
        runtime.get(trigger);
        let owned_runs = Rc::clone(&effect_owned_runs);
        runtime.effect(move |runtime, ()| {
            runtime.get(tick);
            count(&owned_runs);
        });
        if made.replace(true) {
            return;
        }
        let kept_runs = Rc::clone(&effect_kept_runs);
        runtime.scope_in(keeper, move |runtime| {
            runtime.effect(move |runtime, ()| {
                runtime.get(tick);
                count(&kept_runs);
            });
        });
    });
    flush(&mut runtime);
    assert_eq!((kept_runs.get(), owned_runs.get()), (1, 1));

    runtime.set(trigger, 1);
    flush(&mut runtime);
    assert_eq!(
        (kept_runs.get(), owned_runs.get()),
        (1, 2),
        "the run disposed the effect it owned and made a new one; the kept effect stayed as it was"
    );

    runtime.set(tick, 1);
    flush(&mut runtime);
    assert_eq!((kept_runs.get(), owned_runs.get()), (2, 3));

    runtime.dispose(keeper);
    runtime.set(tick, 2);
    flush(&mut runtime);
    assert_eq!(
        (kept_runs.get(), owned_runs.get()),
        (2, 4),
        "the kept effect went with the scope that owned it"
    );
}
