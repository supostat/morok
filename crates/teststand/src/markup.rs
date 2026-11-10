use std::fmt::Write;
use std::time::Duration;

use morok_core::{ElementId, Role};
use morok_style::States;

use crate::inline_style::style_attribute;
use crate::marker_classes::{self, PLACEHOLDER_MARKER};
use crate::scene::Scene;
use crate::scroll_markup::scroll_attributes;

/// Marks the element of a text; the harness page finds the texts by it.
pub(crate) const TEXT_ATTRIBUTE: &str = "data-text";
/// Marks the element of a text the user edits, `line` or `lines`; the harness page finds the
/// edited texts by it.
pub(crate) const EDITED_ATTRIBUTE: &str = "data-edited";
/// Where the caret of a focused edited text stands, and where its selection begins and ends,
/// in UTF-16 code units of the text, as the page counts them.
pub(crate) const CARET_ATTRIBUTE: &str = "data-caret";
pub(crate) const SELECTION_START_ATTRIBUTE: &str = "data-selection-start";
pub(crate) const SELECTION_END_ATTRIBUTE: &str = "data-selection-end";

/// The markup Chrome lays the tree of the scene out from: one `div` per element, in document
/// order, with the classes of the element, the marker classes of its states and the
/// declarations set on it directly. A text holds
/// its characters; an icon is its SVG inline, so that `currentColor` is the color of the
/// element; an image is an `img` with its file as a data URL. An edited text holds its
/// characters, or while it is empty its placeholder; a focused one tells the page where its
/// caret or its selection is.
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
        matches!(
            role,
            Role::Element | Role::Text | Role::Icon | Role::Image | Role::TextInput | Role::TextArea | Role::Scroll
        ),
        "the stand has no markup for {role:?}"
    );
    let shows_placeholder = role.is_editable_text() && tree.text(element).is_empty();
    let mut classes = class_list(scene, element);
    if shows_placeholder {
        classes.push(PLACEHOLDER_MARKER.to_owned());
    }
    let mut attributes = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    let inline_declarations = tree.inline_declarations(element);
    if !inline_declarations.is_empty() {
        let _ = write!(attributes, " style=\"{}\"", style_attribute(inline_declarations));
    }
    attributes.push_str(&scroll_attributes(scene, element));
    match role {
        Role::Icon => {
            let icon = scene.icon_of(element).expect("an icon element shows an icon");
            markup.push_str(&icon.svg.replacen(SVG_TAG, &format!("{SVG_TAG}{attributes}"), 1));
            return;
        }
        Role::Image => {
            let image = scene.image_of(element).expect("an image element shows an image");
            let _ = write!(
                markup,
                "<img{attributes} src=\"data:{};base64,{}\">",
                image.media_type,
                base64(image.encoded)
            );
            return;
        }
        Role::Element | Role::Text | Role::TextInput | Role::TextArea | Role::Scroll => {}
        Role::Canvas => unreachable!("the roles were checked above"),
    }
    markup.push_str("<div");
    markup.push_str(&attributes);
    match role {
        Role::Text => {
            markup.push(' ');
            markup.push_str(TEXT_ATTRIBUTE);
        }
        Role::TextInput | Role::TextArea => write_edited_attributes(scene, element, shows_placeholder, markup),
        Role::Element | Role::Image | Role::Icon | Role::Canvas | Role::Scroll => {}
    }
    markup.push('>');
    if shows_placeholder {
        write_escaped(tree.placeholder(element), markup);
    } else {
        write_escaped(tree.text(element), markup);
    }
    for child in tree.children(element) {
        write_element(scene, *child, markup);
    }
    markup.push_str("</div>");
}

/// Whether the edited text is one line or lines, and for a text shown focused where its caret
/// or its selection is. A placeholder is shown with the caret before it.
fn write_edited_attributes(scene: &Scene, element: ElementId, shows_placeholder: bool, markup: &mut String) {
    let tree = scene.tree();
    let lines = if tree.role(element) == Role::TextArea {
        "lines"
    } else {
        "line"
    };
    let _ = write!(markup, " {EDITED_ATTRIBUTE}=\"{lines}\"");
    if !tree.states(element).contains(States::FOCUS) {
        return;
    }
    let text = tree.text(element);
    let utf16 = |byte: usize| text[..byte].encode_utf16().count();
    let selection = tree.text_selection(element);
    if selection.is_caret() {
        let caret = if shows_placeholder { 0 } else { utf16(selection.head) };
        let _ = write!(markup, " {CARET_ATTRIBUTE}=\"{caret}\"");
    } else {
        let range = selection.range();
        let _ = write!(
            markup,
            " {SELECTION_START_ATTRIBUTE}=\"{}\" {SELECTION_END_ATTRIBUTE}=\"{}\"",
            utf16(range.start),
            utf16(range.end)
        );
    }
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

/// An edited text of the kit keeps its white space as it was typed and shows what fits in its
/// box: one line runs on past the box, lines wrap at its width.
const EDITED_TEXT_STYLE: &str = "[EDITED_ATTRIBUTE] { overflow: hidden; }\n\
                           [EDITED_ATTRIBUTE=\"line\"] { white-space: pre; }\n\
                           [EDITED_ATTRIBUTE=\"lines\"] { white-space: pre-wrap; }";

/// Shows the selection and the caret of a focused edited text. The selection is the page's
/// own, colored by `::selection`. The caret Chrome draws blinks, so the page draws one of its
/// own instead: one CSS pixel wide, where a collapsed range at the caret lies and as high as
/// the range, its edges on device pixels, in the `caret-color` of the text.
const EDITED_TEXT_SCRIPT: &str = "for (const edited of document.querySelectorAll('[EDITED_ATTRIBUTE]')) {\n\
    const text = edited.firstChild;\n\
    if (edited.hasAttribute('SELECTION_START_ATTRIBUTE')) {\n\
    getSelection().setBaseAndExtent(text, Number(edited.getAttribute('SELECTION_START_ATTRIBUTE')), \
    text, Number(edited.getAttribute('SELECTION_END_ATTRIBUTE')));\n\
    }\n\
    if (edited.hasAttribute('CARET_ATTRIBUTE')) {\n\
    const range = document.createRange();\n\
    range.setStart(text, Number(edited.getAttribute('CARET_ATTRIBUTE')));\n\
    range.collapse(true);\n\
    const box = range.getClientRects()[0];\n\
    const snapped = length => Math.round(length * devicePixelRatio) / devicePixelRatio;\n\
    const style = getComputedStyle(edited);\n\
    const color = style.caretColor === 'auto' ? style.color : style.caretColor;\n\
    const caret = document.createElement('span');\n\
    const top = snapped(box.top);\n\
    caret.style.cssText = `position: fixed; left: ${snapped(box.left)}px; top: ${top}px; width: 1px; \
    height: ${snapped(box.bottom) - top}px; background: ${color};`;\n\
    document.body.appendChild(caret);\n\
    }\n\
    }";

/// What the page needs to show the edited texts of the scene as the kit shows them; none for
/// a scene without one, whose page stays as it was before the kit edited text.
pub(crate) fn edited_text_support(scene: &Scene) -> Option<String> {
    let tree = scene.tree();
    let holds_edited_text = scene
        .elements()
        .into_iter()
        .any(|element| tree.role(element).is_editable_text());
    if !holds_edited_text {
        return None;
    }
    let script = EDITED_TEXT_SCRIPT
        .replace("EDITED_ATTRIBUTE", EDITED_ATTRIBUTE)
        .replace("SELECTION_START_ATTRIBUTE", SELECTION_START_ATTRIBUTE)
        .replace("SELECTION_END_ATTRIBUTE", SELECTION_END_ATTRIBUTE)
        .replace("CARET_ATTRIBUTE", CARET_ATTRIBUTE);
    let style = EDITED_TEXT_STYLE.replace("EDITED_ATTRIBUTE", EDITED_ATTRIBUTE);
    Some(format!("<style>\n{style}\n</style><script>\n{script}\n</script>"))
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
