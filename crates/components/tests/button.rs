mod support;

use std::cell::Cell;
use std::rc::Rc;

use morok_components::Button;
use morok_components::styles::CLASSES;
use morok_core::accesskit::{Action, Role as AccessibleRole};
use morok_core::{ElementId, Focusability, Key, Modifiers, Role};
use morok_style::States;
use support::Stage;

/// A stage with one button whose presses are counted.
fn stage_with(button: impl FnOnce(Button) -> Button) -> (Stage, ElementId, Rc<Cell<u32>>) {
    let mut stage = Stage::new();
    let presses = Rc::new(Cell::new(0));
    let button_presses = Rc::clone(&presses);
    let element = stage.mount(move |mount| {
        button(Button::new("Save"))
            .on_press(move |_| button_presses.set(button_presses.get() + 1))
            .mount(mount)
    });
    stage.frame();
    (stage, element, presses)
}

#[test]
fn a_button_is_a_box_of_its_class_around_the_text_of_its_label() {
    let (stage, button, _) = stage_with(|button| button);
    let tree = stage.tree();

    assert_eq!(tree.classes(button), [CLASSES.button]);
    let [label] = tree.children(button) else {
        panic!("a button holds its label alone");
    };
    assert_eq!((tree.role(*label), tree.text(*label)), (Role::Text, "Save"));
    assert_eq!(tree.focusability(button), Focusability::TabStop);
    assert_eq!(
        tree.border_box(button).height.to_bits(),
        28.0_f32.to_bits(),
        "the stylesheet of the button styles it"
    );
}

#[test]
fn the_primary_variant_adds_its_class() {
    let (stage, button, _) = stage_with(Button::primary);

    assert_eq!(stage.tree().classes(button), [CLASSES.button, CLASSES.primary]);
}

#[test]
fn a_button_is_a_node_of_the_button_role_named_by_its_label_that_offers_a_click() {
    let (mut stage, button, _) = stage_with(|button| button);

    let node = stage.accessible_node(button);

    assert_eq!(node.role(), AccessibleRole::Button);
    assert_eq!(node.label(), Some("Save"));
    assert!(node.supports_action(Action::Click) && node.supports_action(Action::Focus));
    assert!(!node.is_disabled());
}

#[test]
fn a_click_presses_the_button() {
    let (mut stage, button, presses) = stage_with(|button| button);

    stage.click(button);

    assert_eq!(presses.get(), 1);
}

#[test]
fn space_and_return_press_the_button_that_has_focus_once_however_long_they_are_held() {
    let (mut stage, button, presses) = stage_with(|button| button);
    assert!(
        !stage.press_key(Key::Space, Modifiers::NONE),
        "without focus no key is used"
    );
    assert!(stage.focus(button));

    assert!(stage.press_key(Key::Space, Modifiers::NONE));
    assert!(stage.press_key(Key::Enter, Modifiers::NONE));
    assert_eq!(presses.get(), 2);

    assert!(
        stage.repeat_key(Key::Space),
        "a held key is used and presses nothing again"
    );
    assert_eq!(presses.get(), 2);

    assert!(
        !stage.press_key(Key::Enter, Modifiers::COMMAND),
        "a key with a modifier held is left to the shortcuts"
    );
    assert!(!stage.press_key(Key::Character('a'), Modifiers::NONE));
    assert_eq!(presses.get(), 2);
}

#[test]
fn an_assistive_technology_presses_the_button() {
    let (mut stage, button, presses) = stage_with(|button| button);

    stage.accessibility_click(button);

    assert_eq!(presses.get(), 1);
}

#[test]
fn a_disabled_button_is_pressed_by_nothing() {
    let (mut stage, button, presses) = stage_with(|button| button.disabled(true));

    assert!(stage.tree().states(button).contains(States::DISABLED));
    assert!(stage.accessible_node(button).is_disabled());
    stage.click(button);
    stage.accessibility_click(button);
    assert!(
        !stage.focus(button),
        "a disabled button takes no focus, so no key reaches it"
    );

    assert_eq!(presses.get(), 0);
}

#[test]
fn a_button_follows_the_signals_of_its_label_and_of_its_disabled_state() {
    let mut stage = Stage::new();
    let label = stage.runtime.signal(String::from("Clean up"));
    let is_busy = stage.runtime.signal(false);
    let presses = Rc::new(Cell::new(0));
    let button_presses = Rc::clone(&presses);
    let button = stage.mount(move |mount| {
        Button::new(label)
            .disabled(is_busy)
            .on_press(move |_| button_presses.set(button_presses.get() + 1))
            .mount(mount)
    });
    stage.frame();
    let label_text = stage.children(button)[0];
    stage.click(button);
    assert_eq!(presses.get(), 1);

    stage.runtime.set(label, String::from("Cleaning up…"));
    stage.runtime.set(is_busy, true);
    stage.frame();

    assert_eq!(
        stage.children(button),
        [label_text],
        "the button and its label are not rebuilt"
    );
    assert_eq!(stage.tree().text(label_text), "Cleaning up…");
    assert_eq!(stage.accessible_node(button).label(), Some("Cleaning up…"));
    assert!(stage.tree().states(button).contains(States::DISABLED));
    stage.click(button);
    assert_eq!(presses.get(), 1);

    stage.runtime.set(is_busy, false);
    stage.frame();
    stage.click(button);
    assert_eq!(presses.get(), 2);
}
