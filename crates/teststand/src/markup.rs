use std::fmt::Write;
use std::time::Duration;

use morok_core::{ElementId, Role};

use crate::marker_classes;
use crate::scene::Scene;

/// Marks the element of a text; the harness page finds the texts by it.
pub(crate) const TEXT_ATTRIBUTE: &str = "data-text";

/// The markup Chrome lays the tree of the scene out from: one `div` per element, in document
/// order, with the classes of the element and the marker classes of its states. A text holds
/// its characters; an icon is its SVG inline, so that `currentColor` is the color of the
/// element; an image is an `img` with its file as a data URL.
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
        matches!(role, Role::Element | Role::Text | Role::Icon | Role::Image),
        "the stand has no markup for {role:?}"
    );
    let classes = class_list(scene, element);
    let class_attribute = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    match role {
        Role::Icon => {
            let icon = scene.icon_of(element).expect("an icon element shows an icon");
            markup.push_str(&icon.svg.replacen(SVG_TAG, &format!("{SVG_TAG}{class_attribute}"), 1));
            return;
        }
        Role::Image => {
            let image = scene.image_of(element).expect("an image element shows an image");
            let _ = write!(
                markup,
                "<img{class_attribute} src=\"data:{};base64,{}\">",
                image.media_type,
                base64(image.encoded)
            );
            return;
        }
        Role::Element | Role::Text => {}
        Role::Canvas | Role::Scroll => unreachable!("the roles were checked above"),
    }
    markup.push_str("<div");
    markup.push_str(&class_attribute);
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

/// The classes of the element with the marker classes of its states, as its markup carries them.
fn class_list(scene: &Scene, element: ElementId) -> Vec<String> {
    scene
        .class_names(element)
        .map(str::to_owned)
        .chain(marker_classes::of_states(scene.tree().states(element)))
        .collect()
}

/// The script that hands the page what the case changes and when it is drawn: the classes of
/// every element after the change, in document order, and the time to draw at.
pub(crate) fn motion_script(changed: &Scene, after: Duration) -> String {
    let classes: Vec<String> = changed
        .elements()
        .into_iter()
        .map(|element| format!("\"{}\"", class_list(changed, element).join(" ")))
        .collect();
    format!(
        "<script>window.{MOTION_VARIABLE} = {{ classes: [{}], milliseconds: {} }};</script>",
        classes.join(", "),
        after.as_millis()
    )
}

/// The global the motion script of a case leaves for the stand.
pub(crate) const MOTION_VARIABLE: &str = "morokMotion";

const SVG_TAG: &str = "<svg";
const BASE64_ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// The standard base64 of the bytes, padded.
fn base64(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let mut word = 0_u32;
        for (position, byte) in chunk.iter().enumerate() {
            word |= u32::from(*byte) << (16 - 8 * position);
        }
        for position in 0..4 {
            if position <= chunk.len() {
                let index = (word >> (18 - 6 * position)) & 0x3f;
                encoded.push(BASE64_ALPHABET[index as usize] as char);
            } else {
                encoded.push('=');
            }
        }
    }
    encoded
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
    use crate::moment::Moment;
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
            component_stylesheets: Vec::new(),
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
                moment: Moment::AtRest,
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
