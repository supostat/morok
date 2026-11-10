mod support;

use std::cell::Cell;
use std::rc::Rc;

use morok_components::styles::CLASSES;
use morok_components::{
    Checkbox, IconButton, KitIcon, ProgressBar, RadioGroup, SegmentedControl, Slider, Spinner, Switch,
};
use morok_core::accesskit::{Action, ActionData, Role as AccessibleRole, Toggled};
use morok_core::{ElementId, Focusability, Key, Modifiers, Role};
use morok_reactive::Signal;
use morok_style::States;
use support::Stage;

/// How far apart two numbers a slider computes may be and still count as one.
const TOLERANCE: f64 = 1e-9;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < TOLERANCE,
        "expected {expected}, the control holds {actual}"
    );
}

/// A stage with one control mounted and laid out.
fn stage_with(component: impl FnOnce(&mut morok_components::view::Mount<'_>) -> ElementId) -> (Stage, ElementId) {
    let mut stage = Stage::new();
    let element = stage.mount(component);
    stage.frame();
    (stage, element)
}

fn toggled(stage: &mut Stage, element: ElementId) -> Option<Toggled> {
    stage.accessible_node(element).toggled()
}

mod check_box {
    use super::*;

    fn check_box(is_disabled: bool) -> (Stage, ElementId, Signal<bool>) {
        let mut stage = Stage::new();
        let is_checked = stage.runtime.signal(false);
        let check_box = stage.mount(move |mount| {
            Checkbox::new(is_checked, "Show hidden files")
                .disabled(is_disabled)
                .mount(mount)
        });
        stage.frame();
        (stage, check_box, is_checked)
    }

    #[test]
    fn a_check_box_is_a_row_of_a_box_with_its_mark_and_the_text_of_its_label() {
        let (mut stage, check_box, _) = check_box(false);
        let tree = stage.tree();

        assert_eq!(tree.classes(check_box), [CLASSES.check]);
        let [indicator, label] = tree.children(check_box) else {
            panic!("a check box holds its box and its label");
        };
        assert_eq!(tree.classes(*indicator), [CLASSES.check_box]);
        let [mark] = tree.children(*indicator) else {
            panic!("the box holds the mark");
        };
        assert_eq!(
            (tree.role(*mark), tree.classes(*mark)),
            (Role::Icon, [CLASSES.check_mark].as_slice())
        );
        assert_eq!(tree.text(*label), "Show hidden files");
        assert_eq!(tree.focusability(check_box), Focusability::TabStop);
        let node = stage.accessible_node(check_box);
        assert_eq!(node.role(), AccessibleRole::CheckBox);
        assert_eq!(node.label(), Some("Show hidden files"));
        assert_eq!(node.toggled(), Some(Toggled::False));
        assert!(node.supports_action(Action::Click));
    }

    #[test]
    fn a_click_space_and_an_assistive_technology_check_and_uncheck_the_box() {
        let (mut stage, check_box, is_checked) = check_box(false);

        stage.click(check_box);
        assert!(stage.runtime.get(is_checked), "a click checks the box");
        stage.frame();
        let indicator = stage.children(check_box)[0];
        assert!(stage.tree().states(check_box).contains(States::CHECKED));
        assert!(
            stage.tree().states(indicator).contains(States::CHECKED),
            "the box is checked with its row"
        );
        assert_eq!(toggled(&mut stage, check_box), Some(Toggled::True));

        assert!(stage.focus(check_box));
        assert!(stage.press_key(Key::Space, Modifiers::NONE));
        assert!(!stage.runtime.get(is_checked), "Space unchecks it");
        assert!(stage.repeat_key(Key::Space), "a held key is used and changes nothing");
        assert!(!stage.runtime.get(is_checked));
        assert!(
            !stage.press_key(Key::Enter, Modifiers::NONE),
            "Return is left to the default button"
        );

        stage.accessibility_click(check_box);
        assert!(stage.runtime.get(is_checked), "an assistive technology checks it");
    }

    #[test]
    fn a_check_box_shows_the_signal_it_is_given() {
        let (mut stage, check_box, is_checked) = check_box(false);

        stage.runtime.set(is_checked, true);
        stage.frame();

        assert!(stage.tree().states(check_box).contains(States::CHECKED));
        assert_eq!(toggled(&mut stage, check_box), Some(Toggled::True));
    }

    #[test]
    fn a_disabled_check_box_changes_by_nothing() {
        let (mut stage, check_box, is_checked) = check_box(true);

        stage.click(check_box);
        stage.accessibility_click(check_box);
        assert!(!stage.focus(check_box), "it takes no focus, so no key reaches it");

        assert!(!stage.runtime.get(is_checked));
        assert!(stage.accessible_node(check_box).is_disabled());
    }
}

mod switch {
    use super::*;

    fn switch(is_disabled: bool) -> (Stage, ElementId, Signal<bool>) {
        let mut stage = Stage::new();
        let is_on = stage.runtime.signal(true);
        let switch = stage.mount(move |mount| Switch::new(is_on, "Sync").disabled(is_disabled).mount(mount));
        stage.frame();
        (stage, switch, is_on)
    }

    #[test]
    fn a_switch_is_a_row_of_a_track_with_its_knob_and_the_text_of_its_label() {
        let (mut stage, switch, _) = switch(false);
        let tree = stage.tree();

        let [track, label] = tree.children(switch) else {
            panic!("a switch holds its track and its label");
        };
        assert_eq!(tree.classes(*track), [CLASSES.switch]);
        let [knob] = tree.children(*track) else {
            panic!("the track holds the knob");
        };
        assert_eq!(tree.classes(*knob), [CLASSES.switch_knob]);
        assert_eq!(tree.text(*label), "Sync");
        assert!(
            tree.states(*track).contains(States::CHECKED),
            "the track shows the switch is on"
        );
        let node = stage.accessible_node(switch);
        assert_eq!(node.role(), AccessibleRole::Switch);
        assert_eq!(node.toggled(), Some(Toggled::True));
    }

    #[test]
    fn a_click_space_and_an_assistive_technology_flip_the_switch() {
        let (mut stage, switch, is_on) = switch(false);

        stage.click(switch);
        assert!(!stage.runtime.get(is_on));
        assert!(stage.focus(switch));
        assert!(stage.press_key(Key::Space, Modifiers::NONE));
        assert!(stage.runtime.get(is_on));
        stage.accessibility_click(switch);
        assert!(!stage.runtime.get(is_on));

        stage.frame();
        let track = stage.children(switch)[0];
        assert!(!stage.tree().states(track).contains(States::CHECKED));
        assert_eq!(toggled(&mut stage, switch), Some(Toggled::False));
    }

    #[test]
    fn a_disabled_switch_changes_by_nothing() {
        let (mut stage, switch, is_on) = switch(true);

        stage.click(switch);
        stage.accessibility_click(switch);

        assert!(!stage.focus(switch));
        assert!(stage.runtime.get(is_on));
    }
}

mod radio_group {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Chart {
        Treemap,
        Sunburst,
        Icicle,
    }

    struct RadioScene {
        stage: Stage,
        group: ElementId,
        radios: Vec<ElementId>,
        chart: Signal<Chart>,
    }

    fn group(chosen: Chart, is_disabled: bool) -> RadioScene {
        let mut stage = Stage::new();
        let chart = stage.runtime.signal(chosen);
        let group = stage.mount(move |mount| {
            RadioGroup::new(chart)
                .option(Chart::Treemap, "Treemap")
                .option(Chart::Sunburst, "Sunburst")
                .option(Chart::Icicle, "Icicle")
                .label("Chart")
                .disabled(is_disabled)
                .mount(mount)
        });
        stage.frame();
        let radios = stage.children(group);
        RadioScene {
            stage,
            group,
            radios,
            chart,
        }
    }

    #[test]
    fn a_group_is_a_column_of_radio_buttons_of_which_the_chosen_one_is_checked() {
        let RadioScene {
            mut stage,
            group,
            radios,
            ..
        } = group(Chart::Sunburst, false);
        let tree = stage.tree();

        assert_eq!(tree.classes(group), [CLASSES.radio_group]);
        assert_eq!(radios.len(), 3);
        assert_eq!(tree.classes(radios[1]), [CLASSES.check, CLASSES.radio]);
        let [radio_box, label] = tree.children(radios[1]) else {
            panic!("a radio button holds its box and its label");
        };
        assert_eq!(tree.classes(*radio_box), [CLASSES.check_box]);
        assert_eq!(tree.classes(tree.children(*radio_box)[0]), [CLASSES.radio_dot]);
        assert_eq!(tree.text(*label), "Sunburst");
        let group_node = stage.accessible_node(group);
        assert_eq!(group_node.role(), AccessibleRole::RadioGroup);
        assert_eq!(group_node.label(), Some("Chart"));
        let toggles: Vec<_> = radios.iter().map(|radio| toggled(&mut stage, *radio)).collect();
        assert_eq!(
            toggles,
            [Some(Toggled::False), Some(Toggled::True), Some(Toggled::False)]
        );
        assert_eq!(stage.accessible_node(radios[0]).role(), AccessibleRole::RadioButton);
    }

    #[test]
    fn the_tab_key_stops_on_the_chosen_radio_button_alone_or_on_the_first_while_none_is() {
        let RadioScene {
            mut stage,
            radios,
            chart,
            ..
        } = group(Chart::Icicle, false);
        let focusabilities = |stage: &Stage| -> Vec<Focusability> {
            radios.iter().map(|radio| stage.tree().focusability(*radio)).collect()
        };

        assert_eq!(
            focusabilities(&stage),
            [Focusability::Focusable, Focusability::Focusable, Focusability::TabStop]
        );

        let mut empty = Stage::new();
        let nothing = empty.runtime.signal(None);
        let group = empty.mount(move |mount| {
            RadioGroup::new(nothing)
                .option(Some(Chart::Treemap), "Treemap")
                .option(Some(Chart::Sunburst), "Sunburst")
                .mount(mount)
        });
        empty.frame();
        let first_and_second: Vec<_> = empty
            .children(group)
            .iter()
            .map(|radio| empty.tree().focusability(*radio))
            .collect();
        assert_eq!(first_and_second, [Focusability::TabStop, Focusability::Focusable]);

        stage.runtime.set(chart, Chart::Treemap);
        stage.frame();
        assert_eq!(
            focusabilities(&stage),
            [Focusability::TabStop, Focusability::Focusable, Focusability::Focusable]
        );
    }

    #[test]
    fn a_click_space_and_an_assistive_technology_choose_a_radio_button() {
        let RadioScene {
            mut stage,
            radios,
            chart,
            ..
        } = group(Chart::Treemap, false);

        stage.click(radios[2]);
        assert_eq!(stage.runtime.get(chart), Chart::Icicle);

        assert!(
            stage.focus(radios[1]),
            "a radio button that is not chosen takes focus all the same"
        );
        assert!(stage.press_key(Key::Space, Modifiers::NONE));
        assert_eq!(stage.runtime.get(chart), Chart::Sunburst);

        stage.accessibility_click(radios[0]);
        assert_eq!(stage.runtime.get(chart), Chart::Treemap);
        stage.frame();
        assert!(stage.tree().states(radios[0]).contains(States::CHECKED));
        assert!(!stage.tree().states(radios[1]).contains(States::CHECKED));
    }

    #[test]
    fn the_arrow_keys_choose_the_next_and_the_previous_radio_button_around_the_ends_and_focus_follows() {
        let RadioScene {
            mut stage,
            radios,
            chart,
            ..
        } = group(Chart::Sunburst, false);
        assert!(stage.focus(radios[1]));

        assert!(stage.press_key(Key::ArrowDown, Modifiers::NONE));
        assert_eq!(stage.runtime.get(chart), Chart::Icicle);
        stage.frame();
        assert_eq!(stage.tree().focused(), Some(radios[2]), "focus follows the choice");

        assert!(stage.press_key(Key::ArrowRight, Modifiers::NONE));
        assert_eq!(
            stage.runtime.get(chart),
            Chart::Treemap,
            "the choice goes around the end"
        );
        stage.frame();
        assert_eq!(stage.tree().focused(), Some(radios[0]));

        assert!(stage.press_key(Key::ArrowUp, Modifiers::NONE));
        assert_eq!(stage.runtime.get(chart), Chart::Icicle);
        assert!(stage.press_key(Key::ArrowLeft, Modifiers::NONE));
        stage.frame();
        assert_eq!(stage.runtime.get(chart), Chart::Sunburst);
        assert_eq!(stage.tree().focused(), Some(radios[1]));
        assert!(
            stage.tree().states(radios[1]).contains(States::FOCUS_VISIBLE),
            "focus moved by the keyboard shows"
        );
    }

    #[test]
    fn a_choice_made_elsewhere_leaves_focus_where_it_is() {
        let RadioScene {
            mut stage,
            radios,
            chart,
            ..
        } = group(Chart::Treemap, false);

        stage.runtime.set(chart, Chart::Icicle);
        stage.frame();

        assert_eq!(stage.tree().focused(), None);
        assert!(stage.tree().states(radios[2]).contains(States::CHECKED));
    }

    #[test]
    fn a_disabled_group_changes_by_nothing() {
        let RadioScene {
            mut stage,
            group,
            radios,
            chart,
        } = group(Chart::Treemap, true);

        stage.click(radios[1]);
        stage.accessibility_click(radios[2]);
        assert!(!stage.focus(radios[0]));

        assert_eq!(stage.runtime.get(chart), Chart::Treemap);
        assert!(stage.tree().states(group).contains(States::DISABLED));
        assert!(stage.accessible_node(radios[1]).is_disabled());
    }
}

mod segmented_control {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Layout {
        List,
        Columns,
        Grid,
    }

    fn control(is_disabled: bool) -> (Stage, ElementId, Signal<Layout>) {
        let mut stage = Stage::new();
        let layout = stage.runtime.signal(Layout::List);
        let control = stage.mount(move |mount| {
            SegmentedControl::new(layout)
                .segment(Layout::List, "List")
                .segment(Layout::Columns, "Columns")
                .segment(Layout::Grid, "Grid")
                .label("Layout")
                .disabled(is_disabled)
                .mount(mount)
        });
        stage.frame();
        (stage, control, layout)
    }

    #[test]
    fn a_segmented_control_is_a_strip_of_segments_of_which_the_chosen_one_is_checked() {
        let (mut stage, control, _) = control(false);
        let segments = stage.children(control);

        assert_eq!(stage.tree().classes(control), [CLASSES.segmented]);
        assert_eq!(stage.texts_of_children(segments[1]), ["Columns"]);
        assert!(
            segments
                .iter()
                .all(|segment| stage.tree().classes(*segment) == [CLASSES.segment])
        );
        assert!(stage.tree().states(segments[0]).contains(States::CHECKED));
        assert_eq!(stage.tree().focusability(segments[0]), Focusability::TabStop);
        assert_eq!(stage.tree().focusability(segments[2]), Focusability::Focusable);
        assert_eq!(stage.accessible_node(control).role(), AccessibleRole::RadioGroup);
        let segment = stage.accessible_node(segments[1]);
        assert_eq!(
            (segment.role(), segment.label(), segment.toggled()),
            (AccessibleRole::RadioButton, Some("Columns"), Some(Toggled::False))
        );
    }

    #[test]
    fn a_click_space_an_assistive_technology_and_the_arrow_keys_choose_a_segment() {
        let (mut stage, control, layout) = control(false);
        let segments = stage.children(control);

        stage.click(segments[2]);
        assert_eq!(stage.runtime.get(layout), Layout::Grid);

        stage.accessibility_click(segments[1]);
        assert_eq!(stage.runtime.get(layout), Layout::Columns);

        assert!(stage.focus(segments[0]));
        assert!(stage.press_key(Key::Space, Modifiers::NONE));
        assert_eq!(stage.runtime.get(layout), Layout::List);

        assert!(stage.press_key(Key::ArrowRight, Modifiers::NONE));
        assert_eq!(stage.runtime.get(layout), Layout::Columns);
        stage.frame();
        assert_eq!(stage.tree().focused(), Some(segments[1]));
        assert!(stage.press_key(Key::ArrowLeft, Modifiers::NONE));
        assert!(stage.press_key(Key::ArrowLeft, Modifiers::NONE));
        assert_eq!(stage.runtime.get(layout), Layout::Grid);
        stage.frame();
        assert_eq!(stage.tree().focused(), Some(segments[2]));
        assert_eq!(toggled(&mut stage, segments[2]), Some(Toggled::True));
    }

    #[test]
    fn a_disabled_segmented_control_changes_by_nothing() {
        let (mut stage, control, layout) = control(true);
        let segments = stage.children(control);

        stage.click(segments[1]);
        stage.accessibility_click(segments[2]);
        assert!(!stage.focus(segments[0]));

        assert_eq!(stage.runtime.get(layout), Layout::List);
    }
}

mod icon_button {
    use super::*;

    #[test]
    fn an_icon_button_is_a_box_around_its_icon_named_by_its_label() {
        let (mut stage, button) = stage_with(|mount| IconButton::new(KitIcon::Sidebar, "Show sidebar").mount(mount));
        let tree = stage.tree();

        assert_eq!(tree.classes(button), [CLASSES.icon_button]);
        let [icon] = tree.children(button) else {
            panic!("an icon button holds its icon alone");
        };
        assert_eq!(
            (tree.role(*icon), tree.classes(*icon)),
            (Role::Icon, [CLASSES.icon_button_icon].as_slice())
        );
        let node = stage.accessible_node(button);
        assert_eq!(
            (node.role(), node.label()),
            (AccessibleRole::Button, Some("Show sidebar"))
        );
        assert_eq!(
            node.toggled(),
            None,
            "a button that is no toggle tells of no checked state"
        );
    }

    #[test]
    fn a_click_space_return_and_an_assistive_technology_press_the_button() {
        let mut stage = Stage::new();
        let presses = Rc::new(Cell::new(0));
        let button_presses = Rc::clone(&presses);
        let button = stage.mount(move |mount| {
            IconButton::new(KitIcon::Trash, "Move to Trash")
                .on_press(move |_| button_presses.set(button_presses.get() + 1))
                .mount(mount)
        });
        stage.frame();

        stage.click(button);
        assert!(stage.focus(button));
        assert!(stage.press_key(Key::Space, Modifiers::NONE));
        assert!(stage.press_key(Key::Enter, Modifiers::NONE));
        stage.accessibility_click(button);

        assert_eq!(presses.get(), 4);
    }

    #[test]
    fn a_press_flips_a_toggle_which_tells_an_assistive_technology_whether_it_is_on() {
        let mut stage = Stage::new();
        let is_grid = stage.runtime.signal(false);
        let button = stage.mount(move |mount| {
            IconButton::new(KitIcon::Grid, "Show as grid")
                .toggled(is_grid)
                .mount(mount)
        });
        stage.frame();
        assert_eq!(toggled(&mut stage, button), Some(Toggled::False));

        stage.click(button);
        assert!(stage.runtime.get(is_grid));
        stage.frame();
        assert!(stage.tree().states(button).contains(States::CHECKED));
        assert_eq!(toggled(&mut stage, button), Some(Toggled::True));

        assert!(stage.focus(button));
        assert!(stage.press_key(Key::Space, Modifiers::NONE));
        assert!(!stage.runtime.get(is_grid));
        stage.accessibility_click(button);
        assert!(stage.runtime.get(is_grid));
    }

    #[test]
    fn a_disabled_icon_button_is_pressed_by_nothing() {
        let mut stage = Stage::new();
        let is_on = stage.runtime.signal(false);
        let button = stage.mount(move |mount| {
            IconButton::new(KitIcon::Gear, "Settings")
                .toggled(is_on)
                .disabled(true)
                .mount(mount)
        });
        stage.frame();

        stage.click(button);
        stage.accessibility_click(button);

        assert!(!stage.focus(button));
        assert!(!stage.runtime.get(is_on));
    }
}

mod slider {
    use super::*;

    fn slider(build: impl FnOnce(Slider) -> Slider + 'static, value: f64) -> (Stage, ElementId, Signal<f64>) {
        let mut stage = Stage::new();
        let level = stage.runtime.signal(value);
        let slider = stage.mount(move |mount| build(Slider::new(level).label("Volume")).mount(mount));
        stage.frame();
        (stage, slider, level)
    }

    #[test]
    fn a_slider_is_a_rail_a_fill_up_to_the_value_and_a_thumb_at_the_value() {
        let (mut stage, slider, _) = slider(|slider| slider, 0.4);
        let tree = stage.tree();

        assert_eq!(tree.classes(slider), [CLASSES.slider]);
        let [rail, fill, thumb] = tree.children(slider) else {
            panic!("a slider holds its rail, its fill and its thumb");
        };
        assert_eq!(tree.classes(*rail), [CLASSES.slider_rail]);
        let slider_box = tree.border_box(slider);
        let fill_box = tree.border_box(*fill);
        let thumb_box = tree.border_box(*thumb);
        assert_eq!(
            slider_box.width.to_bits(),
            220.0_f32.to_bits(),
            "the stylesheet of the slider styles it"
        );
        assert!((fill_box.width - slider_box.width * 0.4).abs() < 0.01);
        assert!(
            (thumb_box.x + thumb_box.width / 2.0 - (slider_box.x + slider_box.width * 0.4)).abs() < 0.01,
            "the middle of the thumb is at the value"
        );
        let node = stage.accessible_node(slider);
        assert_eq!((node.role(), node.label()), (AccessibleRole::Slider, Some("Volume")));
        assert_close(node.numeric_value().expect("a slider reads as a number"), 0.4);
        assert_eq!(
            (
                node.min_numeric_value(),
                node.max_numeric_value(),
                node.numeric_value_step()
            ),
            (Some(0.0), Some(1.0), Some(0.01))
        );
        assert!(
            node.supports_action(Action::Increment)
                && node.supports_action(Action::Decrement)
                && node.supports_action(Action::SetValue)
        );
    }

    #[test]
    fn a_press_sets_the_thumb_down_under_the_pointer_and_a_drag_takes_it_along() {
        let (mut stage, slider, level) = slider(|slider| slider, 0.0);

        stage.click_at(stage.across(slider, 0.25));
        assert_close(stage.runtime.get(level), 0.25);

        let thumb = stage.children(slider)[2];
        stage.frame();
        let from = stage.middle_of(thumb);
        stage.drag(from, &[stage.across(slider, 0.5), stage.across(slider, 0.75)]);
        assert_close(stage.runtime.get(level), 0.75);

        stage.drag(stage.across(slider, 0.6), &[stage.across(slider, 1.5)]);
        assert_close(stage.runtime.get(level), 1.0);

        stage.move_pointer(stage.across(slider, 0.1));
        assert_close(stage.runtime.get(level), 1.0);
    }

    #[test]
    fn the_fill_and_the_thumb_follow_the_value() {
        let (mut stage, slider, level) = slider(|slider| slider, 0.2);
        let [_, fill, thumb] = stage.children(slider)[..] else {
            panic!("a slider holds its rail, its fill and its thumb");
        };

        stage.runtime.set(level, 0.9);
        stage.frame();

        let slider_box = stage.tree().border_box(slider);
        assert!((stage.tree().border_box(fill).width - slider_box.width * 0.9).abs() < 0.01);
        let thumb_box = stage.tree().border_box(thumb);
        assert!((thumb_box.x + thumb_box.width / 2.0 - (slider_box.x + slider_box.width * 0.9)).abs() < 0.01);
        assert_eq!(stage.children(slider), [stage.children(slider)[0], fill, thumb]);
    }

    #[test]
    fn the_keys_move_the_thumb_a_step_a_page_or_to_an_end() {
        let (mut stage, slider, level) = slider(|slider| slider.range(0.0..=100.0, 5.0), 50.0);
        assert!(stage.focus(slider));

        let mut after = |key: Key| {
            assert!(stage.press_key(key, Modifiers::NONE));
            stage.runtime.get(level)
        };
        assert_close(after(Key::ArrowRight), 55.0);
        assert_close(after(Key::ArrowUp), 60.0);
        assert_close(after(Key::ArrowLeft), 55.0);
        assert_close(after(Key::ArrowDown), 50.0);
        assert_close(after(Key::PageUp), 60.0);
        assert_close(after(Key::PageDown), 50.0);
        assert_close(after(Key::End), 100.0);
        assert_close(after(Key::ArrowRight), 100.0);
        assert_close(after(Key::Home), 0.0);
        assert!(
            !stage.press_key(Key::ArrowRight, Modifiers::COMMAND),
            "a key with a modifier is left to the shortcuts"
        );
        assert!(stage.repeat_key(Key::ArrowRight), "a held arrow keeps moving the thumb");
        assert_close(stage.runtime.get(level), 5.0);
    }

    #[test]
    fn a_press_lands_on_the_nearest_step_of_the_range() {
        let (mut stage, slider, level) = slider(|slider| slider.range(0.0..=100.0, 5.0), 0.0);

        stage.click_at(stage.across(slider, 0.33));

        assert_close(stage.runtime.get(level), 35.0);
    }

    #[test]
    fn an_assistive_technology_moves_the_thumb_a_step_or_sets_the_value() {
        let (mut stage, slider, level) = slider(|slider| slider.range(0.0..=10.0, 1.0), 4.0);

        stage.accessibility_action(slider, Action::Increment, None);
        assert_close(stage.runtime.get(level), 5.0);
        stage.accessibility_action(slider, Action::Decrement, None);
        stage.accessibility_action(slider, Action::Decrement, None);
        assert_close(stage.runtime.get(level), 3.0);
        stage.accessibility_action(slider, Action::SetValue, Some(ActionData::NumericValue(7.4)));
        assert_close(stage.runtime.get(level), 7.0);

        stage.frame();
        assert_close(
            stage
                .accessible_node(slider)
                .numeric_value()
                .expect("a slider reads as a number"),
            7.0,
        );
    }

    #[test]
    fn a_disabled_slider_moves_by_nothing() {
        let (mut stage, slider, level) = slider(|slider| slider.disabled(true), 0.5);

        stage.click_at(stage.across(slider, 0.1));
        stage.accessibility_action(slider, Action::Increment, None);
        assert!(!stage.focus(slider));

        assert_close(stage.runtime.get(level), 0.5);
    }
}

mod progress {
    use super::*;

    #[test]
    fn a_progress_bar_is_filled_as_far_as_the_task_is_done_and_reads_it_in_percent() {
        let mut stage = Stage::new();
        let done = stage.runtime.signal(0.62);
        let bar = stage.mount(move |mount| ProgressBar::determinate(done).label("Scanning").mount(mount));
        stage.frame();
        let [fill] = stage.children(bar)[..] else {
            panic!("a progress bar holds its fill");
        };
        let width_of = |stage: &Stage| stage.tree().border_box(fill).width / stage.tree().border_box(bar).width;

        assert_eq!(stage.tree().classes(bar), [CLASSES.progress]);
        assert!((width_of(&stage) - 0.62).abs() < 0.001);
        let node = stage.accessible_node(bar);
        assert_eq!(
            (node.role(), node.label()),
            (AccessibleRole::ProgressIndicator, Some("Scanning"))
        );
        assert_close(
            node.numeric_value()
                .expect("a progress bar that knows reads as a number"),
            62.0,
        );

        stage.runtime.set(done, 1.5);
        stage.frame();
        assert!((width_of(&stage) - 1.0).abs() < 0.001, "the fill stops at the end");
        assert_close(
            stage.accessible_node(bar).numeric_value().expect("still a number"),
            100.0,
        );
    }

    #[test]
    fn a_progress_bar_that_does_not_know_how_far_the_task_has_come_reads_as_no_number() {
        let (mut stage, bar) = stage_with(|mount| ProgressBar::indeterminate().label("Preparing").mount(mount));

        assert_eq!(stage.tree().classes(bar), [CLASSES.progress, CLASSES.indeterminate]);
        let node = stage.accessible_node(bar);
        assert_eq!(node.role(), AccessibleRole::ProgressIndicator);
        assert_eq!(node.numeric_value(), None);
    }

    #[test]
    fn a_spinner_is_a_progress_indicator_named_by_what_is_under_way() {
        let (mut stage, spinner) = stage_with(|mount| Spinner::new().label("Loading").mount(mount));

        assert_eq!(stage.tree().classes(spinner), [CLASSES.spinner]);
        let node = stage.accessible_node(spinner);
        assert_eq!(
            (node.role(), node.label(), node.numeric_value()),
            (AccessibleRole::ProgressIndicator, Some("Loading"), None)
        );
    }
}
