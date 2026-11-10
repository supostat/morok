mod support;

use std::cell::Cell;
use std::rc::Rc;

use morok_components::styles::CLASSES;
use morok_components::view::{ElementEvent, Mount};
use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{ElementId, Focusability, Key, KeyChord, Modifiers, PointerEventKind, Propagation};
use morok_reactive::Signal;
use morok_style::{Declaration, States};
use support::Stage;

fn counter() -> Rc<Cell<u32>> {
    Rc::new(Cell::new(0))
}

fn count(counter: &Rc<Cell<u32>>) {
    counter.set(counter.get() + 1);
}

/// A box of the height of a button that holds the texts of `texts`, one child for each, told
/// apart by the texts themselves.
fn list_of(mount: &mut Mount<'_>, texts: Signal<Vec<String>>, mounted_items: &Rc<Cell<u32>>) -> ElementId {
    let mounted_items = Rc::clone(mounted_items);
    mount
        .element()
        .class(CLASSES.button)
        .for_each(texts, String::clone, move |mount, text| {
            count(&mounted_items);
            mount.text(text.as_str()).id()
        })
        .id()
}

fn texts(of: &[&str]) -> Vec<String> {
    of.iter().map(|text| (*text).to_owned()).collect()
}

#[test]
fn a_component_runs_once_and_its_bound_text_follows_the_signal_on_the_same_element() {
    let mut stage = Stage::new();
    let calls = counter();
    let name = stage.runtime.signal(String::from("draft"));
    let component_calls = Rc::clone(&calls);
    let title = stage.mount(move |mount| {
        count(&component_calls);
        mount.text(name).id()
    });
    stage.frame();
    assert_eq!(stage.tree().text(title), "draft");

    stage.runtime.set(name, String::from("final"));
    stage.frame();

    assert_eq!(stage.tree().text(title), "final");
    assert_eq!(
        calls.get(),
        1,
        "the component is not called again when its signal changes"
    );
    assert_eq!(
        stage.children(stage.root()),
        [title],
        "the text is still the element first built"
    );
}

#[test]
fn a_property_given_a_value_needs_no_effect_and_one_given_a_memo_follows_what_the_memo_reads() {
    let mut stage = Stage::new();
    let fixed = stage.mount(|mount| mount.text("fixed").id());
    assert_eq!(stage.tree().text(fixed), "fixed");
    assert!(!stage.runtime.has_queued_effects());

    let count = stage.runtime.signal(2);
    let derived = stage.mount(move |mount| {
        let label = mount
            .runtime()
            .memo(move |runtime| format!("{} items", runtime.get(count)));
        mount.text(label).id()
    });
    stage.frame();
    assert_eq!(stage.tree().text(derived), "2 items");

    stage.runtime.set(count, 3);
    stage.frame();
    assert_eq!(stage.tree().text(derived), "3 items");
}

#[test]
fn a_bound_class_state_and_inline_declaration_follow_their_signals() {
    let mut stage = Stage::new();
    let is_primary = stage.runtime.signal(false);
    let is_disabled = stage.runtime.signal(true);
    let fade = stage.runtime.signal(Declaration::Opacity(0.25));
    let element = stage.mount(move |mount| {
        mount
            .element()
            .class(CLASSES.button)
            .class_when(CLASSES.primary, is_primary)
            .state(States::DISABLED, is_disabled)
            .state(States::CHECKED, true)
            .inline(fade)
            .id()
    });
    stage.frame();
    assert_eq!(stage.tree().classes(element), [CLASSES.button]);
    assert!(
        stage
            .tree()
            .states(element)
            .contains(States::DISABLED.union(States::CHECKED))
    );
    assert_eq!(
        stage.tree().style(element).effects.opacity.to_bits(),
        0.25_f32.to_bits()
    );

    stage.runtime.set(is_primary, true);
    stage.runtime.set(is_disabled, false);
    stage.runtime.set(fade, Declaration::Opacity(0.75));
    stage.frame();

    assert_eq!(stage.tree().classes(element), [CLASSES.button, CLASSES.primary]);
    let states = stage.tree().states(element);
    assert!(states.contains(States::CHECKED) && !states.contains(States::DISABLED));
    assert_eq!(
        stage.tree().style(element).effects.opacity.to_bits(),
        0.75_f32.to_bits()
    );

    stage.runtime.set(is_primary, false);
    stage.frame();
    assert_eq!(stage.tree().classes(element), [CLASSES.button]);
}

#[test]
fn a_bound_accessible_name_follows_its_signal() {
    let mut stage = Stage::new();
    let name = stage.runtime.signal(String::from("Save"));
    let element = stage.mount(move |mount| {
        mount
            .element()
            .accessible_role(AccessibleRole::Button)
            .accessible_name(name)
            .id()
    });
    stage.frame();
    assert_eq!(stage.accessible_node(element).label(), Some("Save"));

    stage.runtime.set(name, String::from("Save all"));
    stage.frame();
    assert_eq!(stage.accessible_node(element).label(), Some("Save all"));
}

#[test]
fn an_effect_given_the_tree_drives_elements_directly() {
    let mut stage = Stage::new();
    let titles = stage.runtime.signal(texts(&["one", "two"]));
    let mut rows = Vec::new();
    stage.mount(|mount| {
        rows = vec![mount.text("").id(), mount.text("").id()];
        mount.element().child(rows[0]).child(rows[1]).id()
    });
    let pool = rows.clone();
    stage.mount(move |mount| {
        mount.effect(move |runtime, tree| {
            runtime.with(titles, |titles| {
                for (row, title) in pool.iter().zip(titles) {
                    tree.set_text(*row, title);
                }
            });
        });
        mount.element().id()
    });
    stage.frame();
    assert_eq!(stage.tree().text(rows[0]), "one");

    stage.runtime.set(titles, texts(&["three", "four"]));
    stage.frame();
    assert_eq!(
        [stage.tree().text(rows[0]), stage.tree().text(rows[1])],
        ["three", "four"]
    );
}

#[test]
fn show_mounts_its_content_while_the_condition_holds_between_the_children_around_it() {
    let mut stage = Stage::new();
    let is_open = stage.runtime.signal(false);
    let mounted = counter();
    let content_mounted = Rc::clone(&mounted);
    let mut fixed = Vec::new();
    let parent = stage.mount(|mount| {
        let before = mount.text("before").id();
        let after = mount.text("after").id();
        fixed = vec![before, after];
        mount
            .element()
            .child(before)
            .show(is_open, move |mount| {
                count(&content_mounted);
                mount.text("detail").id()
            })
            .child(after)
            .id()
    });
    stage.frame();
    assert_eq!(
        stage.children(parent),
        fixed,
        "nothing is mounted while the condition does not hold"
    );

    stage.runtime.set(is_open, true);
    stage.frame();
    assert_eq!(stage.texts_of_children(parent), ["before", "detail", "after"]);
    let detail = stage.children(parent)[1];

    stage.runtime.set(is_open, true);
    stage.frame();
    assert_eq!(
        stage.children(parent)[1],
        detail,
        "a condition written again keeps the content"
    );
    assert_eq!(mounted.get(), 1);

    stage.runtime.set(is_open, false);
    stage.frame();
    assert_eq!(stage.children(parent), fixed);
    assert!(!stage.tree().contains(detail), "the content is removed from the tree");

    stage.runtime.set(is_open, true);
    stage.frame();
    assert_eq!(stage.texts_of_children(parent), ["before", "detail", "after"]);
    assert_eq!(
        mounted.get(),
        2,
        "the content is mounted anew when the condition holds again"
    );
}

#[test]
fn for_each_keeps_the_element_of_a_key_that_stays_and_mounts_only_the_keys_that_come() {
    let mut stage = Stage::new();
    let names = stage.runtime.signal(texts(&["a", "b", "c"]));
    let mounted = counter();
    let list_mounted = Rc::clone(&mounted);
    let list = stage.mount(move |mount| list_of(mount, names, &list_mounted));
    stage.frame();
    assert_eq!(stage.texts_of_children(list), ["a", "b", "c"]);
    let [a, b, c] = stage.children(list)[..] else {
        panic!("the list holds three children");
    };

    stage.runtime.set(names, texts(&["c", "a", "d", "b"]));
    stage.frame();
    assert_eq!(stage.texts_of_children(list), ["c", "a", "d", "b"]);
    let reordered = stage.children(list);
    assert_eq!(
        [reordered[0], reordered[1], reordered[3]],
        [c, a, b],
        "the items that stay keep their elements wherever they move"
    );
    assert_eq!(mounted.get(), 4, "only the item that came was mounted");

    stage.runtime.set(names, texts(&["a", "d"]));
    stage.frame();
    assert_eq!(stage.children(list), [a, reordered[2]]);
    assert!(!stage.tree().contains(b) && !stage.tree().contains(c));

    stage.runtime.update(names, |names| names.push(String::from("e")));
    stage.frame();
    assert_eq!(stage.texts_of_children(list), ["a", "d", "e"]);
    assert_eq!(stage.children(list)[0], a);
    assert_eq!(mounted.get(), 5);
}

#[test]
fn two_fragments_of_one_parent_keep_their_places_among_its_children() {
    let mut stage = Stage::new();
    let is_open = stage.runtime.signal(true);
    let names = stage.runtime.signal(texts(&["a"]));
    let parent = stage.mount(move |mount| {
        let head = mount.text("head").id();
        let middle = mount.text("middle").id();
        let tail = mount.text("tail").id();
        mount
            .element()
            .child(head)
            .for_each(names, String::clone, |mount, name| mount.text(name.as_str()).id())
            .child(middle)
            .show(is_open, |mount| mount.text("shown").id())
            .child(tail)
            .id()
    });
    stage.frame();
    assert_eq!(
        stage.texts_of_children(parent),
        ["head", "a", "middle", "shown", "tail"]
    );

    stage.runtime.set(names, texts(&["b", "a", "c"]));
    stage.frame();
    assert_eq!(
        stage.texts_of_children(parent),
        ["head", "b", "a", "c", "middle", "shown", "tail"]
    );

    stage.runtime.set(is_open, false);
    stage.runtime.set(names, Vec::new());
    stage.frame();
    assert_eq!(stage.texts_of_children(parent), ["head", "middle", "tail"]);

    stage.runtime.set(is_open, true);
    stage.runtime.set(names, texts(&["z"]));
    stage.frame();
    assert_eq!(
        stage.texts_of_children(parent),
        ["head", "z", "middle", "shown", "tail"]
    );
}

#[test]
fn the_scope_of_a_subtree_goes_with_the_subtree() {
    let mut stage = Stage::new();
    let is_open = stage.runtime.signal(true);
    let tick = stage.runtime.signal(0);
    let outer_runs = counter();
    let inner_runs = counter();
    let (content_outer_runs, content_inner_runs) = (Rc::clone(&outer_runs), Rc::clone(&inner_runs));
    stage.mount(move |mount| {
        mount
            .element()
            .show(is_open, move |mount| {
                let outer_runs = Rc::clone(&content_outer_runs);
                mount.effect(move |runtime, _| {
                    runtime.get(tick);
                    count(&outer_runs);
                });
                let inner_runs = Rc::clone(&content_inner_runs);
                mount
                    .element()
                    .show(is_open, move |mount| {
                        let inner_runs = Rc::clone(&inner_runs);
                        mount.effect(move |runtime, _| {
                            runtime.get(tick);
                            count(&inner_runs);
                        });
                        mount.text("inner").id()
                    })
                    .id()
            })
            .id()
    });
    stage.frame();
    stage.runtime.set(tick, 1);
    stage.frame();
    assert_eq!((outer_runs.get(), inner_runs.get()), (2, 2));

    stage.runtime.set(is_open, false);
    stage.frame();
    stage.runtime.set(tick, 2);
    assert!(
        !stage.runtime.has_queued_effects(),
        "no effect of the removed subtree, or of the subtree inside it, is left to wake"
    );
    stage.frame();
    assert_eq!((outer_runs.get(), inner_runs.get()), (2, 2));
}

#[test]
fn the_scope_of_an_item_goes_with_the_item_and_leaves_the_other_items_running() {
    let mut stage = Stage::new();
    let names = stage.runtime.signal(texts(&["kept", "gone"]));
    let suffix = stage.runtime.signal(String::from("!"));
    let list = stage.mount(move |mount| {
        mount
            .element()
            .for_each(names, String::clone, move |mount, name| {
                let name = name.clone();
                let label = mount
                    .runtime()
                    .memo(move |runtime| runtime.with(suffix, |suffix| format!("{name}{suffix}")));
                mount.text(label).id()
            })
            .id()
    });
    stage.frame();
    assert_eq!(stage.texts_of_children(list), ["kept!", "gone!"]);

    stage.runtime.set(names, texts(&["kept"]));
    stage.frame();
    stage.runtime.set(suffix, String::from("?"));
    stage.frame();

    assert_eq!(stage.texts_of_children(list), ["kept?"]);
}

#[test]
fn closing_a_surface_disposes_what_was_mounted_on_it() {
    let mut stage = Stage::new();
    let name = stage.runtime.signal(String::from("shown twice"));
    let second = stage.open_surface();
    let first_text = stage.mount(move |mount| mount.text(name).id());
    let second_text = stage.mount_on(second, move |mount| mount.text(name).id());
    stage.frame();
    assert_eq!(stage.tree().text(first_text), "shown twice");
    assert_eq!(
        stage.tree_of(second).text(second_text),
        "shown twice",
        "one flush updates every surface a signal is shown on"
    );

    stage.surfaces.close(&mut stage.runtime, second);
    stage.runtime.set(name, String::from("shown once"));
    stage.frame();

    assert_eq!(stage.tree().text(first_text), "shown once");
}

#[test]
fn an_element_hears_of_its_events_and_what_its_handler_writes_shows_after_the_flush() {
    let mut stage = Stage::new();
    let clicks = stage.runtime.signal(0);
    let mut label = None;
    let target = stage.mount(|mount| {
        let shown = mount
            .runtime()
            .memo(move |runtime| format!("clicked {}", runtime.get(clicks)));
        let text = mount.text(shown).id();
        label = Some(text);
        mount
            .element()
            .class(CLASSES.button)
            .on_event(move |runtime, event| match event {
                ElementEvent::Pointer(pointer) if matches!(pointer.kind, PointerEventKind::Clicked { .. }) => {
                    runtime.update(clicks, |clicks| *clicks += 1);
                    Propagation::Stop
                }
                _ => Propagation::Continue,
            })
            .child(text)
            .id()
    });
    let label = label.expect("the component built its label");
    stage.frame();

    stage.click(target);
    assert_eq!(
        stage.tree().text(label),
        "clicked 0",
        "the handler wrote a signal; the tree changes in the flush"
    );
    assert!(stage.runtime.has_queued_effects());
    stage.frame();
    assert_eq!(stage.tree().text(label), "clicked 1");
}

#[test]
fn the_handlers_of_a_removed_subtree_hear_nothing() {
    let mut stage = Stage::new();
    let is_open = stage.runtime.signal(true);
    let heard = counter();
    let content_heard = Rc::clone(&heard);
    let parent = stage.mount(move |mount| {
        mount
            .element()
            .show(is_open, move |mount| {
                let heard = Rc::clone(&content_heard);
                mount
                    .element()
                    .class(CLASSES.button)
                    .on_event(move |_, _| {
                        count(&heard);
                        Propagation::Continue
                    })
                    .id()
            })
            .id()
    });
    stage.frame();
    let target = stage.children(parent)[0];
    let over_target = stage.middle_of(target);
    stage.click_at(over_target);
    let heard_while_mounted = heard.get();
    assert!(heard_while_mounted > 0);

    stage.runtime.set(is_open, false);
    stage.frame();
    stage.click_at(over_target);

    assert_eq!(heard.get(), heard_while_mounted);
}

#[test]
fn a_shortcut_of_an_element_answers_while_focus_is_inside_it_and_goes_with_the_element() {
    let mut stage = Stage::new();
    let is_open = stage.runtime.signal(true);
    let saves = counter();
    let content_saves = Rc::clone(&saves);
    let save = KeyChord {
        key: Key::Character('s'),
        modifiers: Modifiers::COMMAND,
    };
    let parent = stage.mount(move |mount| {
        mount
            .element()
            .show(is_open, move |mount| {
                let saves = Rc::clone(&content_saves);
                let field = mount
                    .element()
                    .class(CLASSES.button)
                    .focusable(Focusability::TabStop)
                    .id();
                mount.element().shortcut(save, move |_| count(&saves)).child(field).id()
            })
            .id()
    });
    stage.frame();
    assert!(
        !stage.press_key(Key::Character('s'), Modifiers::COMMAND),
        "with focus outside the element the shortcut does not answer"
    );

    let panel = stage.children(parent)[0];
    let field = stage.children(panel)[0];
    assert!(stage.focus(field));
    assert!(stage.press_key(Key::Character('s'), Modifiers::COMMAND));
    assert_eq!(saves.get(), 1);

    stage.runtime.set(is_open, false);
    stage.frame();
    assert!(!stage.press_key(Key::Character('s'), Modifiers::COMMAND));
    assert_eq!(saves.get(), 1);
}

#[test]
fn what_a_component_gives_the_root_holds_for_the_surface() {
    let mut stage = Stage::new();
    let closes = counter();
    let root_closes = Rc::clone(&closes);
    let close = KeyChord {
        key: Key::Character('w'),
        modifiers: Modifiers::COMMAND,
    };
    stage.mount(move |mount| {
        mount
            .surface_root()
            .class(CLASSES.primary)
            .shortcut(close, move |_| count(&root_closes));
        mount.element().id()
    });
    stage.frame();

    assert_eq!(stage.tree().classes(stage.root()), [CLASSES.primary]);
    assert!(
        stage.press_key(Key::Character('w'), Modifiers::COMMAND),
        "a shortcut of the root answers while nothing has focus"
    );
    assert_eq!(closes.get(), 1);
}
