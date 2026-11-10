use morok_components::Button;
use morok_core::{ElementId, Size};
use morok_css::SourceFile;
use morok_style::States;

use crate::case::Case;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;
use crate::suites::component_stylesheet;

pub(crate) const NAME: &str = "button";

const COMPONENT_STYLESHEET_PATH: &str = "crates/components/src/button.css";
const STYLESHEET_PATH: &str = "crates/teststand/suites/button.css";
const STYLESHEET: &str = include_str!("../../suites/button.css");

/// `Button` in every state, the default look and the primary variant.
pub(crate) fn suite() -> Suite {
    Suite {
        name: NAME,
        component_stylesheets: vec![component_stylesheet(COMPONENT_STYLESHEET_PATH)],
        stylesheets: vec![SourceFile::new(STYLESHEET_PATH, STYLESHEET)],
        cases: vec![
            case("default-states", default_states),
            case("primary-states", primary_states),
        ],
    }
}

/// A case on a surface that holds a row of buttons and a strip of bare surface under it.
fn case(name: &'static str, build: fn(&mut Scene)) -> Case {
    Case {
        name,
        viewport: Size {
            width: 608.0,
            height: 76.0,
        },
        build,
        moment: Moment::AtRest,
    }
}

fn default_states(scene: &mut Scene) {
    states(scene, |button| button);
}

fn primary_states(scene: &mut Scene) {
    states(scene, Button::primary);
}

/// A row of the buttons of one variant: at rest, under the pointer, pressed, with the focus
/// the keyboard gave, disabled, and disabled with the pointer pressing it.
fn states(scene: &mut Scene, variant: fn(Button) -> Button) {
    let stage = scene.add(scene.root(), &["button-stage"]);
    let row = scene.add(stage, &["button-row"]);
    let pressed = States::HOVER.union(States::ACTIVE);
    button(scene, row, variant(Button::new("Rest")), States::NONE);
    button(scene, row, variant(Button::new("Hover")), States::HOVER);
    button(scene, row, variant(Button::new("Pressed")), pressed);
    button(
        scene,
        row,
        variant(Button::new("Focus")),
        States::FOCUS.union(States::FOCUS_VISIBLE),
    );
    button(
        scene,
        row,
        variant(Button::new("Disabled")).disabled(true),
        States::NONE,
    );
    button(
        scene,
        row,
        variant(Button::new("Disabled, pressed")).disabled(true),
        pressed,
    );
}

/// Mounts the button in the row and puts it in the states the tree would drive from input.
fn button(scene: &mut Scene, row: ElementId, button: Button, driven_states: States) {
    let element = scene.mount(row, |mount| button.mount(mount));
    scene.set_state(element, driven_states);
}
