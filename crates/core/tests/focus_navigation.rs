mod support;

use morok_core::{
    ElementId, Focusability, Key, KeyChord, KeyEvent, KeyPhase, Modifiers, PointerEventKind, Role, ShortcutScope,
};
use morok_style::States;
use support::{InputLog, Scene};

const FORM: &str = ".form { display: flex; gap: 10px; padding: 10px; width: 600px; }\n\
    .group { display: flex; gap: 10px; }\n\
    .field { width: 80px; height: 30px; background-color: var(--color-surface); }\n\
    .field:focus-visible { outline: 2px solid var(--color-focus); }\n\
    .label { width: 80px; height: 30px; }\n\
    .gone { display: none; }";

/// A form of 600 pixels: a field, a group of two fields, a label that takes no focus and a
/// last field, each 80 pixels wide and 10 apart, after the first frame.
struct FormScene {
    scene: Scene,
    form: ElementId,
    name: ElementId,
    group: ElementId,
    street: ElementId,
    city: ElementId,
    label: ElementId,
    notes: ElementId,
}

impl FormScene {
    fn new() -> Self {
        let mut scene = Scene::new(FORM);
        let form = scene.add(scene.root(), Role::Element, &["form"]);
        let name = scene.add(form, Role::Element, &["field"]);
        let group = scene.add(form, Role::Element, &["group"]);
        let street = scene.add(group, Role::Element, &["field"]);
        let city = scene.add(group, Role::Element, &["field"]);
        let label = scene.add(form, Role::Element, &["label"]);
        let notes = scene.add(form, Role::Element, &["field"]);
        for field in [name, street, city, notes] {
            scene.tree.set_focusability(field, Focusability::TabStop);
        }
        scene.frame();
        Self {
            scene,
            form,
            name,
            group,
            street,
            city,
            label,
            notes,
        }
    }

    fn tab(&mut self) -> bool {
        self.scene
            .press_key(Key::Tab, Modifiers::NONE, &mut InputLog::default())
    }

    fn shift_tab(&mut self) -> bool {
        self.scene
            .press_key(Key::Tab, Modifiers::SHIFT, &mut InputLog::default())
    }

    fn focused(&self) -> Option<ElementId> {
        self.scene.tree.focused()
    }

    fn has(&self, element: ElementId, state: States) -> bool {
        self.scene.tree.states(element).contains(state)
    }

    /// Presses and releases the primary button over the middle of the element.
    fn click(&mut self, element: ElementId) {
        let border_box = self.scene.border_box(element);
        let (x, y) = (
            border_box.x + border_box.width / 2.0,
            border_box.y + border_box.height / 2.0,
        );
        let mut log = InputLog::default();
        self.scene.press(x, y, &mut log);
        self.scene.release(x, y, &mut log);
    }
}

fn save_chord() -> KeyChord {
    KeyChord {
        key: Key::Character('s'),
        modifiers: Modifiers::COMMAND,
    }
}

#[test]
fn tab_walks_the_tab_stops_in_tree_order_and_starts_over_after_the_last() {
    let mut form = FormScene::new();
    let mut visited = Vec::new();

    for _ in 0..5 {
        assert!(form.tab());
        visited.push(form.focused().expect("a tab stop has focus"));
    }

    assert_eq!(visited, [form.name, form.street, form.city, form.notes, form.name]);
}

#[test]
fn shift_tab_walks_the_tab_stops_backwards() {
    let mut form = FormScene::new();
    let mut visited = Vec::new();

    for _ in 0..5 {
        assert!(form.shift_tab());
        visited.push(form.focused().expect("a tab stop has focus"));
    }

    assert_eq!(visited, [form.notes, form.city, form.street, form.name, form.notes]);
}

#[test]
fn a_container_that_is_a_tab_stop_comes_before_its_content() {
    let mut form = FormScene::new();
    form.scene.tree.set_focusability(form.group, Focusability::TabStop);
    assert!(form.scene.tree.focus(form.name));

    form.tab();
    assert_eq!(form.focused(), Some(form.group));
    form.tab();
    assert_eq!(form.focused(), Some(form.street));
    form.shift_tab();
    form.shift_tab();
    assert_eq!(form.focused(), Some(form.name));
}

#[test]
fn tab_passes_by_what_is_only_focusable_disabled_or_not_rendered() {
    let mut form = FormScene::new();
    form.scene.tree.set_focusability(form.street, Focusability::Focusable);
    form.scene.tree.set_state(form.city, States::DISABLED, true);
    form.scene.add_classes(form.notes, &["gone"]);
    form.scene.frame();

    form.tab();
    assert_eq!(form.focused(), Some(form.name));
    assert!(!form.tab(), "there is no other stop to go to");
    assert_eq!(form.focused(), Some(form.name));
}

#[test]
fn tab_passes_by_everything_inside_an_element_that_is_not_rendered() {
    let mut form = FormScene::new();
    form.scene.add_classes(form.group, &["gone"]);
    form.scene.frame();
    assert!(form.scene.tree.focus(form.name));

    form.tab();

    assert_eq!(form.focused(), Some(form.notes));
}

#[test]
fn with_no_tab_stop_tab_is_not_used() {
    let mut scene = Scene::new(FORM);
    scene.add(scene.root(), Role::Element, &["label"]);
    scene.frame();

    assert!(!scene.press_key(Key::Tab, Modifiers::NONE, &mut InputLog::default()));
    assert_eq!(scene.tree.focused(), None);
}

#[test]
fn an_element_takes_focus_only_as_its_focusability_says() {
    let mut form = FormScene::new();
    form.scene.tree.set_focusability(form.street, Focusability::Focusable);

    assert!(
        form.scene.tree.focus(form.street),
        "a focusable element takes focus when asked"
    );
    assert!(
        !form.scene.tree.focus(form.label),
        "an element that is not focusable takes none"
    );
    assert_eq!(form.focused(), Some(form.street));

    form.scene.tree.set_state(form.city, States::DISABLED, true);
    assert!(!form.scene.tree.focus(form.city), "a disabled element takes none");
    let detached = form.scene.tree.create(Role::Element);
    form.scene.tree.set_focusability(detached, Focusability::TabStop);
    assert!(
        !form.scene.tree.focus(detached),
        "an element outside the tree takes none"
    );
}

#[test]
fn focus_is_shown_to_the_keyboard_and_not_to_the_pointer() {
    let mut form = FormScene::new();

    form.tab();
    assert!(form.has(form.name, States::FOCUS) && form.has(form.name, States::FOCUS_VISIBLE));

    form.click(form.city);
    assert_eq!(form.focused(), Some(form.city));
    assert!(form.has(form.city, States::FOCUS));
    assert!(!form.has(form.city, States::FOCUS_VISIBLE));
    assert!(!form.has(form.name, States::FOCUS) && !form.has(form.name, States::FOCUS_VISIBLE));

    form.scene
        .press_key(Key::Character('c'), Modifiers::COMMAND, &mut InputLog::default());
    assert!(
        !form.has(form.city, States::FOCUS_VISIBLE),
        "a command tells nothing of the keyboard"
    );

    form.scene
        .press_key(Key::ArrowDown, Modifiers::NONE, &mut InputLog::default());
    assert!(
        form.has(form.city, States::FOCUS_VISIBLE),
        "a key shows the focus the pointer gave"
    );

    form.click(form.label);
    assert_eq!(
        form.focused(),
        Some(form.city),
        "a press on nothing focusable leaves focus"
    );
    assert!(!form.has(form.city, States::FOCUS_VISIBLE));
}

#[test]
fn focus_asked_for_shows_as_the_last_input_does() {
    let mut form = FormScene::new();

    assert!(form.scene.tree.focus(form.name));
    assert!(
        form.has(form.name, States::FOCUS_VISIBLE),
        "before any input focus is shown"
    );

    form.click(form.label);
    assert!(form.scene.tree.focus(form.notes));
    assert!(form.has(form.notes, States::FOCUS) && !form.has(form.notes, States::FOCUS_VISIBLE));
}

#[test]
fn a_focused_element_is_restyled_by_its_focus_rule() {
    let mut form = FormScene::new();

    form.tab();
    form.scene.frame();

    assert!(form.scene.tree.style(form.name).paint.used_outline_width() > 0.0);
    assert!(form.scene.tree.style(form.street).paint.used_outline_width() <= 0.0);
}

#[test]
fn a_press_gives_focus_to_the_innermost_focusable_element_around_it() {
    let mut form = FormScene::new();
    let caption = form.scene.add_text(form.street, "Street");
    form.scene.frame();

    form.click(caption);

    assert_eq!(form.focused(), Some(form.street));
}

#[test]
fn focus_within_marks_the_focused_element_and_everything_around_it() {
    let mut form = FormScene::new();
    let root = form.scene.root();

    assert!(form.scene.tree.focus(form.street));
    for around in [form.street, form.group, form.form, root] {
        assert!(form.has(around, States::FOCUS_WITHIN));
    }
    assert!(!form.has(form.city, States::FOCUS_WITHIN) && !form.has(form.name, States::FOCUS_WITHIN));

    assert!(form.scene.tree.focus(form.notes));
    assert!(!form.has(form.street, States::FOCUS_WITHIN) && !form.has(form.group, States::FOCUS_WITHIN));
    assert!(form.has(form.notes, States::FOCUS_WITHIN) && form.has(form.form, States::FOCUS_WITHIN));

    form.scene.tree.blur();
    assert_eq!(form.focused(), None);
    for element in [form.notes, form.form, root] {
        assert_eq!(form.scene.tree.states(element), States::NONE);
    }
}

#[test]
fn focus_does_not_stay_on_what_leaves_the_tree_is_disabled_or_is_hidden() {
    let mut form = FormScene::new();
    let root = form.scene.root();

    assert!(form.scene.tree.focus(form.street));
    form.scene.tree.remove(form.group);
    assert_eq!(form.focused(), None);
    assert!(!form.has(form.form, States::FOCUS_WITHIN) && !form.has(root, States::FOCUS_WITHIN));

    assert!(form.scene.tree.focus(form.name));
    form.scene.tree.set_state(form.name, States::DISABLED, true);
    assert_eq!(form.focused(), None);
    assert!(!form.has(form.name, States::FOCUS));

    assert!(form.scene.tree.focus(form.notes));
    form.scene.tree.set_focusability(form.notes, Focusability::NotFocusable);
    assert_eq!(form.focused(), None);

    form.scene.tree.set_focusability(form.notes, Focusability::TabStop);
    assert!(form.scene.tree.focus(form.notes));
    form.scene.add_classes(form.form, &["gone"]);
    form.scene.frame();
    assert_eq!(form.focused(), None, "an element that is not rendered keeps no focus");
}

#[test]
fn a_trap_keeps_tab_and_focus_inside_its_element() {
    let mut form = FormScene::new();
    assert!(form.scene.tree.focus(form.name));

    form.scene.tree.trap_focus(form.group);
    assert_eq!(form.focused(), None, "focus outside the trap is dropped");

    let mut visited = Vec::new();
    for _ in 0..3 {
        form.tab();
        visited.push(form.focused().expect("a tab stop has focus"));
    }
    assert_eq!(visited, [form.street, form.city, form.street]);
    form.shift_tab();
    assert_eq!(form.focused(), Some(form.city));

    assert!(
        !form.scene.tree.focus(form.notes),
        "nothing outside the trap takes focus"
    );
    form.click(form.name);
    assert_eq!(
        form.focused(),
        Some(form.city),
        "a press outside the trap moves no focus"
    );
}

#[test]
fn releasing_a_trap_gives_focus_back_to_where_it_was() {
    let mut form = FormScene::new();
    assert!(form.scene.tree.focus(form.notes));
    form.scene.tree.trap_focus(form.group);
    form.tab();

    form.scene.tree.release_focus_trap(form.group);

    assert_eq!(form.focused(), Some(form.notes));
    form.tab();
    assert_eq!(form.focused(), Some(form.name), "tab walks the whole tree again");
}

#[test]
fn of_two_traps_the_one_set_last_holds() {
    let mut form = FormScene::new();
    form.scene.tree.trap_focus(form.form);
    assert!(form.scene.tree.focus(form.name));
    form.scene.tree.trap_focus(form.group);

    form.tab();
    form.tab();
    form.tab();
    assert_eq!(form.focused(), Some(form.street));

    form.scene.tree.release_focus_trap(form.group);
    assert_eq!(form.focused(), Some(form.name));
    assert!(!form.scene.tree.focus(form.scene.root()), "the outer trap still holds");
}

#[test]
fn a_trap_goes_with_its_element_when_the_element_leaves_the_tree() {
    let mut form = FormScene::new();
    form.scene.tree.trap_focus(form.group);

    form.scene.tree.remove(form.group);

    assert!(form.scene.tree.focus(form.notes));
    form.tab();
    assert_eq!(form.focused(), Some(form.name));
}

#[test]
fn a_key_is_offered_to_the_focused_element_and_then_to_each_element_around_it() {
    let mut form = FormScene::new();
    let root = form.scene.root();
    assert!(form.scene.tree.focus(form.street));
    let mut log = InputLog::default();

    let used = form.scene.press_key(Key::Enter, Modifiers::NONE, &mut log);

    assert!(!used);
    assert_eq!(log.key_receivers(), [form.street, form.group, form.form, root]);
}

#[test]
fn an_element_that_uses_a_key_ends_its_way() {
    let mut form = FormScene::new();
    assert!(form.scene.tree.focus(form.street));
    let mut log = InputLog::default();
    log.consumers.push(form.group);

    let used = form.scene.press_key(Key::Tab, Modifiers::NONE, &mut log);

    assert!(used);
    assert_eq!(log.key_receivers(), [form.street, form.group]);
    assert_eq!(
        form.focused(),
        Some(form.street),
        "the key was used, so it moved no focus"
    );
}

#[test]
fn with_no_focus_a_key_is_offered_to_the_root_and_a_release_is_routed_like_a_press() {
    let mut form = FormScene::new();
    let root = form.scene.root();
    let mut log = InputLog::default();
    let released = KeyEvent {
        phase: KeyPhase::Released,
        key: Key::Escape,
        modifiers: Modifiers::NONE,
        is_repeat: false,
    };

    form.scene.tree.key_input(&released, &mut log);

    assert_eq!(log.key_events, [(root, KeyPhase::Released, Key::Escape)]);
}

#[test]
fn a_shortcut_of_an_element_answers_only_with_focus_inside_the_element() {
    let mut form = FormScene::new();
    let save = form
        .scene
        .tree
        .register_shortcut(ShortcutScope::Element(form.group), save_chord());

    assert!(form.scene.tree.focus(form.name));
    let mut log = InputLog::default();
    assert!(!form.scene.press_key(Key::Character('s'), Modifiers::COMMAND, &mut log));
    assert_eq!(log.shortcuts, []);

    assert!(form.scene.tree.focus(form.city));
    let mut log = InputLog::default();
    assert!(form.scene.press_key(Key::Character('s'), Modifiers::COMMAND, &mut log));
    assert_eq!(log.shortcuts, [save]);
    assert_eq!(
        log.key_receivers(),
        [form.city, form.group],
        "the way ends at the shortcut"
    );

    let mut log = InputLog::default();
    assert!(!form.scene.press_key(Key::Character('s'), Modifiers::NONE, &mut log));
    assert_eq!(log.shortcuts, [], "the chord is the key with its modifiers");
}

#[test]
fn a_shortcut_of_the_surface_answers_from_anywhere_and_after_those_of_the_elements() {
    let mut form = FormScene::new();
    let of_surface = form.scene.tree.register_shortcut(ShortcutScope::Surface, save_chord());
    let of_form = form
        .scene
        .tree
        .register_shortcut(ShortcutScope::Element(form.form), save_chord());
    let of_group = form
        .scene
        .tree
        .register_shortcut(ShortcutScope::Element(form.group), save_chord());

    let mut log = InputLog::default();
    form.scene.press_key(Key::Character('s'), Modifiers::COMMAND, &mut log);
    assert_eq!(log.shortcuts, [of_surface], "with no focus the surface answers");

    assert!(form.scene.tree.focus(form.street));
    let mut log = InputLog::default();
    form.scene.press_key(Key::Character('s'), Modifiers::COMMAND, &mut log);
    assert_eq!(log.shortcuts, [of_group], "the innermost scope around focus answers");

    form.scene.tree.unregister_shortcut(of_group);
    let mut log = InputLog::default();
    form.scene.press_key(Key::Character('s'), Modifiers::COMMAND, &mut log);
    assert_eq!(log.shortcuts, [of_form]);

    form.scene.tree.remove(form.form);
    let mut log = InputLog::default();
    form.scene.press_key(Key::Character('s'), Modifiers::COMMAND, &mut log);
    assert_eq!(log.shortcuts, [of_surface], "a shortcut goes with its element");
}

#[test]
fn of_two_shortcuts_with_one_chord_in_one_scope_the_later_answers() {
    let mut form = FormScene::new();
    form.scene.tree.register_shortcut(ShortcutScope::Surface, save_chord());
    let later = form.scene.tree.register_shortcut(ShortcutScope::Surface, save_chord());
    let mut log = InputLog::default();

    form.scene.press_key(Key::Character('s'), Modifiers::COMMAND, &mut log);

    assert_eq!(log.shortcuts, [later]);
}

#[test]
fn an_element_that_uses_the_key_keeps_it_from_the_shortcuts() {
    let mut form = FormScene::new();
    form.scene.tree.register_shortcut(ShortcutScope::Surface, save_chord());
    assert!(form.scene.tree.focus(form.street));
    let mut log = InputLog::default();
    log.consumers.push(form.street);

    assert!(form.scene.press_key(Key::Character('s'), Modifiers::COMMAND, &mut log));

    assert_eq!(log.shortcuts, []);
}

#[test]
fn a_shortcut_answers_to_the_press_of_its_chord_and_not_to_the_release() {
    let mut form = FormScene::new();
    form.scene.tree.register_shortcut(ShortcutScope::Surface, save_chord());
    let mut log = InputLog::default();
    let released = KeyEvent {
        phase: KeyPhase::Released,
        key: Key::Character('s'),
        modifiers: Modifiers::COMMAND,
        is_repeat: false,
    };

    assert!(!form.scene.tree.key_input(&released, &mut log));

    assert_eq!(log.shortcuts, []);
}

#[test]
fn a_press_that_gives_focus_is_told_to_the_elements_first() {
    let mut form = FormScene::new();
    let mut log = InputLog::default();
    let border_box = form.scene.border_box(form.city);

    form.scene.press(border_box.x + 5.0, border_box.y + 5.0, &mut log);

    assert_eq!(form.focused(), Some(form.city));
    assert_eq!(log.receivers_of(PointerEventKind::Entered).last(), Some(&form.city));
}
