use morok_core::{ElementId, Role};

use crate::marker_classes;
use crate::scene::Scene;

/// Marks the element of a text; the harness page finds the texts by it.
pub(crate) const TEXT_ATTRIBUTE: &str = "data-text";

/// The markup Chrome lays the tree of the scene out from: one `div` per element, in document
/// order, with the classes of the element and the marker classes of its states. A text holds
/// its characters.
///
/// # Panics
///
/// When the tree holds an element the stand has no markup for.
pub(crate) fn of(scene: &Scene) -> String {
    let mut markup = String::new();
    write_element(scene, scene.root(), &mut markup);
    markup
}

fn write_element(scene: &Scene, element: ElementId, markup: &mut String) {
    let tree = scene.tree();
    let role = tree.role(element);
    assert!(
        matches!(role, Role::Element | Role::Text),
        "the stand has no markup for {role:?}"
    );
    let classes: Vec<String> = scene
        .class_names(element)
        .map(str::to_owned)
        .chain(marker_classes::of_states(tree.states(element)))
        .collect();
    markup.push_str("<div");
    if !classes.is_empty() {
        markup.push_str(" class=\"");
        markup.push_str(&classes.join(" "));
        markup.push('"');
    }
    if role == Role::Text {
        markup.push(' ');
        markup.push_str(TEXT_ATTRIBUTE);
    }
    markup.push('>');
    write_escaped(tree.text(element), markup);
    for child in tree.children(element) {
        write_element(scene, *child, markup);
    }
    markup.push_str("</div>");
}

fn write_escaped(text: &str, markup: &mut String) {
    for character in text.chars() {
        match character {
            '&' => markup.push_str("&amp;"),
            '<' => markup.push_str("&lt;"),
            '>' => markup.push_str("&gt;"),
            other => markup.push(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use morok_core::Size;
    use morok_css::SourceFile;
    use morok_style::{ColorScheme, States};

    use crate::case::Case;
    use crate::compiled_suite::CompiledSuite;
    use crate::scene::Scene;
    use crate::suite::Suite;

    fn label_in_a_button(scene: &mut Scene) {
        let button = scene.add(scene.root(), &["button", "primary"]);
        scene.set_state(button, States::HOVER.union(States::FOCUS_VISIBLE));
        scene.add_text(button, &["label"], "Save & <close>");
        scene.add(scene.root(), &["button"]);
    }

    #[test]
    fn every_element_is_a_div_with_its_classes_the_markers_of_its_states_and_its_text() {
        let suite = Suite {
            name: "markup",
            stylesheets: vec![SourceFile::new(
                "markup.css",
                ".button { height: 28px; } .button.primary:hover { height: 30px; } .label { height: 12px; }",
            )],
            cases: vec![Case {
                name: "button",
                viewport: Size {
                    width: 100.0,
                    height: 40.0,
                },
                build: label_in_a_button,
            }],
        };
        let compiled = CompiledSuite::compile(&suite).expect("the stylesheet compiles");
        let scene = compiled.scene(&suite.cases[0], ColorScheme::Light);

        assert_eq!(
            super::of(&scene),
            "<div>\
             <div class=\"button primary state--hover state--focus-visible\">\
             <div class=\"label\" data-text>Save &amp; &lt;close&gt;</div>\
             </div>\
             <div class=\"button\"></div>\
             </div>"
        );
        assert_eq!(
            scene.node_names(),
            ["#0 root", "#1 .button.primary", "#2 .label", "#3 .button"]
        );
    }
}
