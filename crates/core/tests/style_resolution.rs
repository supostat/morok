mod support;

use std::borrow::Cow;

use morok_core::{Corners, ResolvedLength, ResolvedLengthOrAuto, Role, Sides};
use morok_style::{
    Color, ColorValue, Declaration, Display, LengthPercentage, LengthPercentageAuto, LineStyle, Pixels, States,
};
use support::Scene;

const TRANSPARENT: Color = Color {
    red: 0,
    green: 0,
    blue: 0,
    alpha: 0.0,
};
const BLACK: Color = Color {
    red: 0,
    green: 0,
    blue: 0,
    alpha: 1.0,
};

fn pixels(pixels: f32) -> ResolvedLength {
    ResolvedLength::pixels(pixels)
}

#[test]
fn a_class_rule_styles_the_element_with_the_values_of_its_tokens() {
    let mut scene = Scene::new(
        ".card { display: flex; gap: var(--space-2); padding: var(--space-4) var(--space-3); \
         background-color: var(--color-surface); border-radius: var(--radius-md); }",
    );
    let card = scene.add(scene.root(), Role::Element, &["card"]);

    scene.frame();

    let style = scene.tree.style(card);
    assert_eq!(style.layout.display, Display::Flex);
    assert_eq!(style.layout.column_gap, pixels(8.0));
    assert_eq!(style.layout.row_gap, pixels(8.0));
    assert_eq!(
        style.layout.padding,
        Sides {
            top: pixels(16.0),
            right: pixels(12.0),
            bottom: pixels(16.0),
            left: pixels(12.0),
        }
    );
    assert_eq!(style.paint.background_color, scene.light_color("color-surface"));
    assert_eq!(
        style.paint.border_radius,
        Corners {
            top_left: 8.0,
            top_right: 8.0,
            bottom_right: 8.0,
            bottom_left: 8.0,
        }
    );
}

#[test]
fn an_element_without_rules_has_the_initial_values_of_the_reset() {
    let mut scene = Scene::new(".card { padding: var(--space-4); background-color: var(--color-surface); }");
    let card = scene.add(scene.root(), Role::Element, &["card"]);
    let plain = scene.add(card, Role::Element, &[]);

    scene.frame();

    let style = scene.tree.style(plain);
    assert_eq!(style.layout.display, Display::Block);
    assert_eq!(style.layout.padding, Sides::uniform(pixels(0.0)));
    assert_eq!(style.layout.margin, Sides::uniform(ResolvedLengthOrAuto::ZERO));
    assert_eq!(style.layout.width, ResolvedLengthOrAuto::Auto);
    assert_eq!(style.layout.used_border_width(), Sides::uniform(0.0));
    assert_eq!(style.paint.background_color, TRANSPARENT);
    assert_eq!(style.paint.color, BLACK);
    assert_eq!(style.paint.outline_style, LineStyle::None);
}

#[test]
fn of_two_rules_of_equal_specificity_the_later_wins() {
    let mut scene = Scene::new(
        ".card { background-color: var(--color-surface); width: 10px; }\n\
         .card { background-color: var(--color-accent); }",
    );
    let card = scene.add(scene.root(), Role::Element, &["card"]);

    scene.frame();

    let style = scene.tree.style(card);
    assert_eq!(style.paint.background_color, scene.light_color("color-accent"));
    assert_eq!(style.layout.width, ResolvedLengthOrAuto::Length(pixels(10.0)));
}

#[test]
fn a_more_specific_rule_wins_over_a_later_less_specific_one() {
    let mut scene = Scene::new(
        ".button.primary { background-color: var(--color-accent); }\n\
         .button { background-color: var(--color-surface); height: 28px; }",
    );
    let primary = scene.add(scene.root(), Role::Element, &["button", "primary"]);
    let plain = scene.add(scene.root(), Role::Element, &["button"]);

    scene.frame();

    assert_eq!(
        scene.tree.style(primary).paint.background_color,
        scene.light_color("color-accent")
    );
    assert_eq!(
        scene.tree.style(primary).layout.height,
        ResolvedLengthOrAuto::Length(pixels(28.0))
    );
    assert_eq!(
        scene.tree.style(plain).paint.background_color,
        scene.light_color("color-surface")
    );
}

#[test]
fn the_order_of_the_classes_on_an_element_does_not_change_the_cascade() {
    let mut scene = Scene::new(
        ".primary { background-color: var(--color-accent); }\n\
         .button { background-color: var(--color-surface); }",
    );
    let first = scene.add(scene.root(), Role::Element, &["button", "primary"]);
    let second = scene.add(scene.root(), Role::Element, &["primary", "button"]);

    scene.frame();

    let surface = scene.light_color("color-surface");
    assert_eq!(scene.tree.style(first).paint.background_color, surface);
    assert_eq!(scene.tree.style(second).paint.background_color, surface);
}

#[test]
fn a_state_overlay_applies_only_while_the_state_is_on() {
    let mut scene = Scene::new(
        ".button { background-color: var(--color-surface); }\n\
         .button:hover { background-color: var(--color-surface-2); }",
    );
    let button = scene.add(scene.root(), Role::Element, &["button"]);
    scene.frame();
    assert_eq!(
        scene.tree.style(button).paint.background_color,
        scene.light_color("color-surface")
    );

    scene.tree.set_state(button, States::HOVER, true);
    scene.frame();
    assert_eq!(
        scene.tree.style(button).paint.background_color,
        scene.light_color("color-surface-2")
    );

    scene.tree.set_state(button, States::HOVER, false);
    scene.frame();
    assert_eq!(
        scene.tree.style(button).paint.background_color,
        scene.light_color("color-surface")
    );
}

#[test]
fn overlays_of_several_states_apply_in_cascade_order() {
    let mut scene = Scene::new(
        ".button:active { background-color: var(--color-accent-active); }\n\
         .button:hover { background-color: var(--color-accent-hover); }\n\
         .button:hover:active { color: var(--color-text-on-accent); }\n\
         .button { background-color: var(--color-accent); color: var(--color-text); }",
    );
    let button = scene.add(scene.root(), Role::Element, &["button"]);

    scene.tree.set_state(button, States::HOVER.union(States::ACTIVE), true);
    scene.frame();

    let style = scene.tree.style(button);
    assert_eq!(style.paint.background_color, scene.light_color("color-accent-hover"));
    assert_eq!(style.paint.color, scene.light_color("color-text-on-accent"));
}

#[test]
fn a_child_rule_styles_only_the_children_of_a_matching_parent() {
    let mut scene = Scene::new(
        ".button { margin-left: 0; }\n\
         .toolbar > .button { margin-left: var(--space-1); }",
    );
    let toolbar = scene.add(scene.root(), Role::Element, &["toolbar"]);
    let child = scene.add(toolbar, Role::Element, &["button"]);
    let wrapper = scene.add(toolbar, Role::Element, &[]);
    let grandchild = scene.add(wrapper, Role::Element, &["button"]);
    let elsewhere = scene.add(scene.root(), Role::Element, &["button"]);

    scene.frame();

    let margin_left = |element| scene.tree.style(element).layout.margin.left;
    assert_eq!(margin_left(child), ResolvedLengthOrAuto::Length(pixels(4.0)));
    assert_eq!(margin_left(grandchild), ResolvedLengthOrAuto::ZERO);
    assert_eq!(margin_left(elsewhere), ResolvedLengthOrAuto::ZERO);
}

#[test]
fn a_child_rule_follows_the_state_and_the_classes_of_the_parent() {
    let mut scene = Scene::new(
        ".table:hover > .row { background-color: var(--color-surface-2); }\n\
         .table.dense > .row { height: 20px; }",
    );
    let table = scene.add(scene.root(), Role::Element, &["table"]);
    let row = scene.add(table, Role::Element, &["row"]);
    scene.frame();
    assert_eq!(scene.tree.style(row).paint.background_color, TRANSPARENT);
    assert_eq!(scene.tree.style(row).layout.height, ResolvedLengthOrAuto::Auto);

    scene.tree.set_state(table, States::HOVER, true);
    let dense = scene.class("dense");
    scene.tree.add_class(table, dense);
    scene.frame();
    assert_eq!(
        scene.tree.style(row).paint.background_color,
        scene.light_color("color-surface-2")
    );
    assert_eq!(
        scene.tree.style(row).layout.height,
        ResolvedLengthOrAuto::Length(pixels(20.0))
    );

    scene.tree.remove_class(table, dense);
    scene.frame();
    assert_eq!(scene.tree.style(row).layout.height, ResolvedLengthOrAuto::Auto);
}

#[test]
fn a_state_flip_restyles_the_element_and_only_the_children_its_child_rules_name() {
    let mut scene = Scene::new(
        ".toolbar:hover > .button { background-color: var(--color-surface-2); }\n\
         .toolbar:active { background-color: var(--color-surface-sunken); }\n\
         .label { width: 40px; }",
    );
    let toolbar = scene.add(scene.root(), Role::Element, &["toolbar"]);
    let first_button = scene.add(toolbar, Role::Element, &["button"]);
    let second_button = scene.add(toolbar, Role::Element, &["button"]);
    scene.add(toolbar, Role::Element, &["label"]);
    scene.add(first_button, Role::Element, &["button"]);
    let first_frame = scene.frame();
    assert_eq!(first_frame.resolved, 6);

    scene.tree.set_state(toolbar, States::HOVER, true);
    let hovered = scene.frame();

    assert_eq!(hovered.resolved, 3);
    let surface_2 = scene.light_color("color-surface-2");
    assert_eq!(scene.tree.style(first_button).paint.background_color, surface_2);
    assert_eq!(scene.tree.style(second_button).paint.background_color, surface_2);

    scene.tree.set_state(toolbar, States::ACTIVE, true);
    let pressed = scene.frame();

    assert_eq!(pressed.resolved, 1);
    assert_eq!(
        scene.tree.style(toolbar).paint.background_color,
        scene.light_color("color-surface-sunken")
    );
}

#[test]
fn setting_a_state_to_what_it_already_is_restyles_nothing() {
    let mut scene = Scene::new(".button:hover { background-color: var(--color-surface-2); }");
    let button = scene.add(scene.root(), Role::Element, &["button"]);
    scene.tree.set_state(button, States::HOVER, true);
    scene.frame();

    scene.tree.set_state(button, States::HOVER, true);
    scene.tree.set_state(button, States::ACTIVE, false);

    assert_eq!(scene.frame().resolved, 0);
}

#[test]
fn color_inherits_from_the_parent_until_an_element_declares_its_own() {
    let mut scene = Scene::new(
        ".app { color: var(--color-text); }\n\
         .muted { color: var(--color-text-2); }",
    );
    let app = scene.add(scene.root(), Role::Element, &["app"]);
    let child = scene.add(app, Role::Element, &[]);
    let muted = scene.add(child, Role::Element, &["muted"]);
    let text = scene.add_text(muted, "inherits");

    scene.frame();

    let text_color = scene.light_color("color-text");
    let muted_color = scene.light_color("color-text-2");
    assert_eq!(scene.tree.style(app).paint.color, text_color);
    assert_eq!(scene.tree.style(child).paint.color, text_color);
    assert_eq!(scene.tree.style(muted).paint.color, muted_color);
    assert_eq!(scene.tree.style(text).paint.color, muted_color);
}

#[test]
fn currentcolor_is_the_color_of_the_element_and_inherits_as_the_color_keyword() {
    let mut scene = Scene::new(
        ".app { color: var(--color-accent); }\n\
         .chip { border: 1px solid currentColor; background-color: currentColor; outline: 2px solid; }\n\
         .same { color: currentColor; border: 1px solid; }",
    );
    let app = scene.add(scene.root(), Role::Element, &["app"]);
    let chip = scene.add(app, Role::Element, &["chip"]);
    let same = scene.add(app, Role::Element, &["same"]);

    scene.frame();

    let accent = scene.light_color("color-accent");
    let chip_style = scene.tree.style(chip);
    assert_eq!(chip_style.paint.border_color, Sides::uniform(accent));
    assert_eq!(chip_style.paint.background_color, accent);
    assert_eq!(chip_style.paint.outline_color, accent);
    assert_eq!(scene.tree.style(same).paint.color, accent);
    assert_eq!(scene.tree.style(same).paint.border_color, Sides::uniform(accent));
}

#[test]
fn a_change_of_an_inherited_color_restyles_the_descendants_that_inherit_it() {
    let mut scene = Scene::new(
        ".item { color: var(--color-text); }\n\
         .item:disabled { color: var(--color-text-3); }\n\
         .badge { color: var(--color-text-on-accent); }",
    );
    let item = scene.add(scene.root(), Role::Element, &["item"]);
    let label = scene.add(item, Role::Element, &[]);
    let label_text = scene.add_text(label, "label");
    let badge = scene.add(item, Role::Element, &["badge"]);
    let badge_text = scene.add_text(badge, "7");
    scene.frame();

    scene.tree.set_state(item, States::DISABLED, true);
    let disabled = scene.frame();

    let dimmed = scene.light_color("color-text-3");
    assert_eq!(scene.tree.style(label).paint.color, dimmed);
    assert_eq!(scene.tree.style(label_text).paint.color, dimmed);
    let on_accent = scene.light_color("color-text-on-accent");
    assert_eq!(scene.tree.style(badge).paint.color, on_accent);
    assert_eq!(scene.tree.style(badge_text).paint.color, on_accent);
    assert_eq!(
        disabled.resolved, 4,
        "the item, its label, the text of the label and the badge"
    );
}

#[test]
fn a_border_takes_space_only_when_it_has_a_style() {
    let mut scene = Scene::new(
        ".widths { border-width: 2px; }\n\
         .styled { border-style: solid; }\n\
         .framed { border-top: 4px solid var(--color-line); }",
    );
    let widths = scene.add(scene.root(), Role::Element, &["widths"]);
    let styled = scene.add(scene.root(), Role::Element, &["styled"]);
    let framed = scene.add(scene.root(), Role::Element, &["framed"]);

    scene.frame();

    assert_eq!(scene.tree.style(widths).layout.used_border_width(), Sides::uniform(0.0));
    assert_eq!(scene.tree.style(styled).layout.used_border_width(), Sides::uniform(3.0));
    assert_eq!(
        scene.tree.style(styled).layout.border_style,
        Sides::uniform(LineStyle::Solid)
    );
    assert_eq!(
        scene.tree.style(framed).layout.used_border_width(),
        Sides {
            top: 4.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    );
    assert_eq!(
        scene.tree.style(framed).paint.border_color.top,
        scene.light_color("color-line")
    );
}

#[test]
fn an_inline_declaration_overrides_every_rule_until_it_is_cleared() {
    let mut scene = Scene::new(
        ".bar { width: 10px; background-color: var(--color-accent); }\n\
         .bar:hover { width: 20px; }",
    );
    let bar = scene.add(scene.root(), Role::Element, &["bar"]);
    scene.tree.set_state(bar, States::HOVER, true);
    let width = |pixels: f32| {
        Declaration::Width(LengthPercentageAuto::LengthPercentage(LengthPercentage {
            pixels: Pixels(pixels),
            fraction: None,
            tokens: Cow::Borrowed(&[]),
        }))
    };

    scene.tree.set_inline(bar, width(64.0));
    scene
        .tree
        .set_inline(bar, Declaration::BackgroundColor(ColorValue::Transparent));
    scene.frame();
    assert_eq!(
        scene.tree.style(bar).layout.width,
        ResolvedLengthOrAuto::Length(pixels(64.0))
    );
    assert_eq!(scene.tree.style(bar).paint.background_color, TRANSPARENT);

    scene.tree.set_inline(bar, width(72.0));
    scene.frame();
    assert_eq!(
        scene.tree.style(bar).layout.width,
        ResolvedLengthOrAuto::Length(pixels(72.0))
    );
    assert_eq!(
        scene.tree.inline_declarations(bar),
        [width(72.0), Declaration::BackgroundColor(ColorValue::Transparent)],
        "a property set again keeps its place"
    );

    scene.tree.clear_inline(bar);
    scene.frame();
    assert!(scene.tree.inline_declarations(bar).is_empty());
    assert_eq!(
        scene.tree.style(bar).layout.width,
        ResolvedLengthOrAuto::Length(pixels(20.0))
    );
    assert_eq!(
        scene.tree.style(bar).paint.background_color,
        scene.light_color("color-accent")
    );
}

#[test]
fn switching_the_theme_restyles_every_element_against_the_other_table() {
    let mut scene = Scene::new(
        ".app { color: var(--color-text); background-color: var(--color-bg); padding: var(--space-4); }\n\
         .card { background-color: var(--color-surface); border: 1px solid var(--color-line); }",
    );
    let app = scene.add(scene.root(), Role::Element, &["app"]);
    let card = scene.add(app, Role::Element, &["card"]);
    let text = scene.add_text(card, "themed");
    scene.frame();
    assert_eq!(
        scene.tree.style(card).paint.background_color,
        scene.light_color("color-surface")
    );
    assert_ne!(scene.light_color("color-surface"), scene.dark_color("color-surface"));

    scene.tree.set_theme(scene.dark_theme());
    let switched = scene.frame();

    assert_eq!(switched.resolved, 4);
    assert_eq!(
        scene.tree.style(app).paint.background_color,
        scene.dark_color("color-bg")
    );
    assert_eq!(
        scene.tree.style(card).paint.background_color,
        scene.dark_color("color-surface")
    );
    assert_eq!(
        scene.tree.style(card).paint.border_color.top,
        scene.dark_color("color-line")
    );
    assert_eq!(scene.tree.style(text).paint.color, scene.dark_color("color-text"));
    assert_eq!(scene.tree.style(app).layout.padding, Sides::uniform(pixels(16.0)));

    scene.tree.set_theme(scene.light_theme());
    scene.frame();

    assert_eq!(
        scene.tree.style(card).paint.background_color,
        scene.light_color("color-surface")
    );
    assert_eq!(scene.tree.style(text).paint.color, scene.light_color("color-text"));
}

#[test]
fn a_subtree_is_styled_when_it_joins_the_tree_and_by_the_parent_it_joins() {
    let mut scene = Scene::new(
        ".app { color: var(--color-text); }\n\
         .sidebar { color: var(--color-text-2); }\n\
         .sidebar > .entry { padding-left: var(--space-2); }",
    );
    let app = scene.add(scene.root(), Role::Element, &["app"]);
    let sidebar = scene.add(scene.root(), Role::Element, &["sidebar"]);
    let entry = scene.tree.create(Role::Element);
    scene.add_classes(entry, &["entry"]);
    let entry_text = scene.add_text(entry, "entry");
    scene.frame();

    scene.tree.append_child(app, entry);
    let joined = scene.frame();

    assert_eq!(joined.resolved, 2);
    assert_eq!(
        scene.tree.style(entry_text).paint.color,
        scene.light_color("color-text")
    );
    assert_eq!(scene.tree.style(entry).layout.padding.left, pixels(0.0));

    scene.tree.detach(entry);
    scene.tree.append_child(sidebar, entry);
    scene.frame();

    assert_eq!(
        scene.tree.style(entry_text).paint.color,
        scene.light_color("color-text-2")
    );
    assert_eq!(scene.tree.style(entry).layout.padding.left, pixels(8.0));
}
