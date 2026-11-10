use std::time::Duration;

use morok_components::view::Mount;
use morok_components::{
    Checkbox, IconButton, KitIcon, ProgressBar, RadioGroup, SegmentedControl, Slider, Spinner, Switch,
};
use morok_core::{ElementId, Size};
use morok_css::SourceFile;
use morok_style::States;

use crate::case::Case;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;
use crate::suites::component_stylesheet;

pub(crate) const NAME: &str = "controls";

const COMPONENT_STYLESHEET_PATHS: [&str; 8] = [
    "crates/components/src/checkbox.css",
    "crates/components/src/icon_button.css",
    "crates/components/src/progress_bar.css",
    "crates/components/src/radio_group.css",
    "crates/components/src/segmented_control.css",
    "crates/components/src/slider.css",
    "crates/components/src/spinner.css",
    "crates/components/src/switch.css",
];
const STYLESHEET_PATH: &str = "crates/teststand/suites/controls.css";
const STYLESHEET: &str = include_str!("../../suites/controls.css");

/// The states the tree gives a control the pointer presses.
const PRESSED: States = States::HOVER.union(States::ACTIVE);
/// The states the tree gives a control the keyboard focused.
const KEYBOARD_FOCUS: States = States::FOCUS.union(States::FOCUS_VISIBLE);

/// `IconButton`, `Checkbox`, `RadioGroup`, `Switch`, `Slider`, `SegmentedControl`,
/// `ProgressBar` and `Spinner` in every state; the indicators that run are drawn early and late
/// in their runs, on no end of an iteration.
pub(crate) fn suite() -> Suite {
    Suite {
        name: NAME,
        component_stylesheets: COMPONENT_STYLESHEET_PATHS
            .iter()
            .map(|path| component_stylesheet(path))
            .collect(),
        stylesheets: vec![SourceFile::new(STYLESHEET_PATH, STYLESHEET)],
        cases: vec![
            at_rest("icon-button-states", 440.0, 60.0, icon_button_states),
            at_rest("checkbox-states", 540.0, 96.0, checkbox_states),
            at_rest("radio-states", 540.0, 88.0, radio_states),
            at_rest("switch-states", 620.0, 96.0, switch_states),
            at_rest("slider-states", 760.0, 120.0, slider_states),
            at_rest("segmented-control-states", 440.0, 104.0, segmented_control_states),
            at_rest("progress-bar-states", 260.0, 84.0, progress_bar_states),
            running("busy-indicators-early", 300),
            running("busy-indicators-late", 1000),
        ],
    }
}

fn at_rest(name: &'static str, width: f32, height: f32, build: fn(&mut Scene)) -> Case {
    Case {
        name,
        viewport: Size { width, height },
        build,
        moment: Moment::AtRest,
    }
}

/// The indicators that run, drawn `milliseconds` after the first frame.
fn running(name: &'static str, milliseconds: u64) -> Case {
    Case {
        name,
        viewport: Size {
            width: 300.0,
            height: 52.0,
        },
        build: busy_indicators,
        moment: Moment::InMotion {
            change: |_| {},
            after: Duration::from_millis(milliseconds),
        },
    }
}

/// The rows of a case on its stage; every row lays its controls out side by side.
fn rows(scene: &mut Scene) -> ElementId {
    let stage = scene.add(scene.root(), &["controls-stage"]);
    scene.add(stage, &["controls-rows"])
}

fn row(scene: &mut Scene, rows: ElementId) -> ElementId {
    scene.add(rows, &["controls-row"])
}

/// Mounts the control in the row and puts it in the states the tree would drive from input.
fn control_in(
    scene: &mut Scene,
    row: ElementId,
    driven_states: States,
    control: impl FnOnce(&mut Mount<'_>) -> ElementId,
) -> ElementId {
    let element = scene.mount(row, control);
    scene.set_state(element, driven_states);
    element
}

/// At rest, under the pointer, pressed, with the focus the keyboard gave, toggled on at rest
/// and under the pointer, disabled, and disabled and toggled on with the pointer pressing it.
fn icon_button_states(scene: &mut Scene) {
    let rows = rows(scene);
    let row = row(scene, rows);
    let plain = |icon: KitIcon, is_disabled: bool| {
        move |mount: &mut Mount<'_>| IconButton::new(icon, "Action").disabled(is_disabled).mount(mount)
    };
    let toggled_on = |icon: KitIcon, is_disabled: bool| {
        move |mount: &mut Mount<'_>| {
            let is_on = mount.runtime().signal(true);
            IconButton::new(icon, "Toggle")
                .toggled(is_on)
                .disabled(is_disabled)
                .mount(mount)
        }
    };
    control_in(scene, row, States::NONE, plain(KitIcon::Sidebar, false));
    control_in(scene, row, States::HOVER, plain(KitIcon::Grid, false));
    control_in(scene, row, PRESSED, plain(KitIcon::Search, false));
    control_in(scene, row, KEYBOARD_FOCUS, plain(KitIcon::Gear, false));
    control_in(scene, row, States::NONE, toggled_on(KitIcon::Star, false));
    control_in(scene, row, States::HOVER, toggled_on(KitIcon::Folder, false));
    control_in(scene, row, States::NONE, plain(KitIcon::Trash, true));
    control_in(scene, row, PRESSED, toggled_on(KitIcon::Plus, true));
}

/// A row of check boxes or switches of one look in their states: at rest, pressed, with the
/// focus the keyboard gave, disabled, and disabled with the pointer pressing it. `control`
/// builds one, given its label and whether it is disabled.
fn check_states(scene: &mut Scene, rows: ElementId, control: impl Fn(&mut Mount<'_>, &str, bool) -> ElementId) {
    let row = row(scene, rows);
    for (label, is_disabled, driven_states) in [
        ("Rest", false, States::NONE),
        ("Pressed", false, PRESSED),
        ("Focus", false, KEYBOARD_FOCUS),
        ("Disabled", true, States::NONE),
        ("Disabled, pressed", true, PRESSED),
    ] {
        control_in(scene, row, driven_states, |mount| control(mount, label, is_disabled));
    }
}

/// Unchecked boxes over checked ones.
fn checkbox_states(scene: &mut Scene) {
    let rows = rows(scene);
    for is_checked in [false, true] {
        check_states(scene, rows, |mount, label, is_disabled| {
            let is_checked = mount.runtime().signal(is_checked);
            Checkbox::new(is_checked, label).disabled(is_disabled).mount(mount)
        });
    }
}

/// Switches that are off over switches that are on.
fn switch_states(scene: &mut Scene) {
    let rows = rows(scene);
    for is_on in [false, true] {
        check_states(scene, rows, |mount, label, is_disabled| {
            let is_on = mount.runtime().signal(is_on);
            Switch::new(is_on, label).disabled(is_disabled).mount(mount)
        });
    }
}

/// Groups of two radio buttons, the first chosen: at rest, both pressed, the chosen one with
/// the focus the keyboard gave, disabled, and disabled with both pressed.
fn radio_states(scene: &mut Scene) {
    let rows = rows(scene);
    let row = row(scene, rows);
    for (is_disabled, driven_states, are_both_driven) in [
        (false, States::NONE, true),
        (false, PRESSED, true),
        (false, KEYBOARD_FOCUS, false),
        (true, States::NONE, true),
        (true, PRESSED, true),
    ] {
        let group = scene.mount(row, |mount| {
            let chart = mount.runtime().signal("treemap");
            RadioGroup::new(chart)
                .option("treemap", "Treemap")
                .option("sunburst", "Sunburst")
                .disabled(is_disabled)
                .mount(mount)
        });
        let radios = scene.tree().children(group).to_vec();
        let driven = if are_both_driven { &radios[..] } else { &radios[..1] };
        for radio in driven {
            scene.set_state(*radio, driven_states);
        }
    }
}

/// Sliders at the ends and in between at rest; under the pointer, pressed and with the focus
/// the keyboard gave; disabled, and disabled with the pointer pressing it.
fn slider_states(scene: &mut Scene) {
    let rows = rows(scene);
    let slider = |value: f64, is_disabled: bool| {
        move |mount: &mut Mount<'_>| {
            let value = mount.runtime().signal(value);
            Slider::new(value).disabled(is_disabled).mount(mount)
        }
    };
    let at_rest = row(scene, rows);
    for value in [0.0, 0.4, 1.0] {
        control_in(scene, at_rest, States::NONE, slider(value, false));
    }
    let driven = row(scene, rows);
    control_in(scene, driven, States::HOVER, slider(0.4, false));
    control_in(scene, driven, PRESSED, slider(0.4, false));
    control_in(scene, driven, KEYBOARD_FOCUS, slider(0.6, false));
    let disabled = row(scene, rows);
    control_in(scene, disabled, States::NONE, slider(0.4, true));
    control_in(scene, disabled, PRESSED, slider(0.4, true));
}

/// Controls of three segments: at rest; with one segment under the pointer and one pressed;
/// with the focus the keyboard gave on the chosen segment; disabled, with the chosen segment
/// under the pointer and another pressed.
fn segmented_control_states(scene: &mut Scene) {
    let rows = rows(scene);
    let control = |chosen: &'static str, is_disabled: bool| {
        move |mount: &mut Mount<'_>| {
            let layout = mount.runtime().signal(chosen);
            SegmentedControl::new(layout)
                .segment("list", "List")
                .segment("columns", "Columns")
                .segment("grid", "Grid")
                .disabled(is_disabled)
                .mount(mount)
        }
    };
    let segments_of = |scene: &Scene, control: ElementId| scene.tree().children(control).to_vec();
    let first = row(scene, rows);
    control_in(scene, first, States::NONE, control("list", false));
    let pointed = control_in(scene, first, States::NONE, control("list", false));
    let segments = segments_of(scene, pointed);
    scene.set_state(segments[1], States::HOVER);
    scene.set_state(segments[2], PRESSED);
    let second = row(scene, rows);
    let focused = control_in(scene, second, States::NONE, control("columns", false));
    let segments = segments_of(scene, focused);
    scene.set_state(segments[1], KEYBOARD_FOCUS);
    let disabled = control_in(scene, second, States::NONE, control("grid", true));
    let segments = segments_of(scene, disabled);
    scene.set_state(segments[2], States::HOVER);
    scene.set_state(segments[0], PRESSED);
}

/// Bars that have not begun, are under way and are done.
fn progress_bar_states(scene: &mut Scene) {
    let rows = rows(scene);
    for done in [0.0, 0.62, 1.0] {
        let row = row(scene, rows);
        control_in(scene, row, States::NONE, |mount| {
            ProgressBar::determinate(done).mount(mount)
        });
    }
}

/// A bar that does not know how far its task has come beside a spinner.
fn busy_indicators(scene: &mut Scene) {
    let rows = rows(scene);
    let row = row(scene, rows);
    control_in(scene, row, States::NONE, |mount| {
        ProgressBar::indeterminate().mount(mount)
    });
    control_in(scene, row, States::NONE, |mount| Spinner::new().mount(mount));
}
