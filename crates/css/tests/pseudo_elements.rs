mod support;

use morok_style::{ColorValue, Declaration, States};
use support::{STYLESHEET_PATH, color, declarations, rejected, style_sheet};

/// A stylesheet with the one diagnostic it must produce: its message and its place on line 1.
struct Rejection {
    stylesheet: &'static str,
    message: &'static str,
    column: usize,
}

fn assert_rejected(rejections: &[Rejection]) {
    for rejection in rejections {
        let diagnostics = rejected(rejection.stylesheet);

        let reported: Vec<_> = diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.message.as_str(),
                    diagnostic.path.to_str(),
                    diagnostic.line,
                    diagnostic.column,
                )
            })
            .collect();
        assert_eq!(
            reported,
            [(rejection.message, Some(STYLESHEET_PATH), 1, rejection.column)],
            "{}",
            rejection.stylesheet
        );
    }
}

#[test]
fn the_color_of_a_placeholder_is_a_declaration_of_its_element() {
    let style_sheet = style_sheet(".field::placeholder { color: var(--color-text-3); }");

    let rule = &style_sheet.rules[0];
    assert_eq!(
        rule.selector.subject.classes.as_ref(),
        [style_sheet.class_named("field").unwrap()]
    );
    assert_eq!(rule.selector.parent, None);
    assert_eq!(
        rule.declarations.as_ref(),
        [Declaration::PlaceholderColor(color("color-text-3"))]
    );
}

#[test]
fn the_background_of_a_selection_is_a_declaration_of_its_element() {
    let style_sheet = style_sheet(".field::selection { background-color: var(--color-selection); }");

    assert_eq!(
        style_sheet.rules[0].declarations.as_ref(),
        [Declaration::SelectionBackgroundColor(color("color-selection"))]
    );
}

#[test]
fn a_pseudo_element_follows_the_classes_and_the_states_of_its_element() {
    let style_sheet = style_sheet(
        ".field { color: var(--color-text); }\n\
         .row > .field.search:disabled::placeholder { color: var(--color-text-2); }",
    );

    let rule = &style_sheet.rules[1];
    assert_eq!(rule.selector.subject.classes.len(), 2);
    assert_eq!(rule.selector.subject.states, States::DISABLED);
    assert!(rule.selector.parent.is_some());
    assert_eq!(
        rule.declarations.as_ref(),
        [Declaration::PlaceholderColor(color("color-text-2"))]
    );
}

#[test]
fn rules_of_one_pseudo_element_share_a_block() {
    let style_sheet = style_sheet(".field::selection, .area::selection { background-color: currentColor; }");

    assert_eq!(style_sheet.rules.len(), 2);
    for rule in style_sheet.rules.iter() {
        assert_eq!(
            rule.declarations.as_ref(),
            [Declaration::SelectionBackgroundColor(ColorValue::CurrentColor)]
        );
    }
}

#[test]
fn the_color_of_the_caret_is_a_color_or_the_color_of_the_text() {
    assert_eq!(
        declarations("caret-color: var(--color-accent)"),
        [Declaration::CaretColor(color("color-accent"))]
    );
    assert_eq!(
        declarations("caret-color: auto"),
        [Declaration::CaretColor(ColorValue::CurrentColor)]
    );
    assert_eq!(
        declarations("caret-color: transparent"),
        [Declaration::CaretColor(ColorValue::Transparent)]
    );
}

#[test]
fn what_is_outside_the_subset_of_pseudo_elements_is_rejected_where_it_stands() {
    assert_rejected(&[
        Rejection {
            stylesheet: ".a::before { display: block; }",
            message: "the pseudo-element `::before` is not supported",
            column: 3,
        },
        Rejection {
            stylesheet: ".a::placeholder { font-size: var(--text-md); }",
            message: "`font-size` is not supported in `::placeholder`",
            column: 19,
        },
        Rejection {
            stylesheet: ".a::placeholder { background-color: var(--color-bg); }",
            message: "`background-color` is not supported in `::placeholder`",
            column: 19,
        },
        Rejection {
            stylesheet: ".a::selection { background: var(--color-selection); }",
            message: "`background` is not supported in `::selection`",
            column: 17,
        },
        Rejection {
            stylesheet: ".a::selection { opacity: 0.5; }",
            message: "`opacity` is not supported in `::selection`",
            column: 17,
        },
        Rejection {
            stylesheet: ".a::selection { color: var(--color-text); }",
            message: "`color` is not supported in `::selection`",
            column: 17,
        },
        Rejection {
            stylesheet: ".a::selection.b { background-color: var(--color-selection); }",
            message: "nothing follows `::selection` in a selector",
            column: 14,
        },
        Rejection {
            stylesheet: ".a::selection:hover { background-color: var(--color-selection); }",
            message: "nothing follows `::selection` in a selector",
            column: 14,
        },
        Rejection {
            stylesheet: ".a::selection > .b { color: var(--color-text); }",
            message: "a pseudo-element has no children to select",
            column: 1,
        },
        Rejection {
            stylesheet: ".a::placeholder, .b { color: var(--color-text); }",
            message: "a rule of a pseudo-element is shared with other selectors",
            column: 1,
        },
        Rejection {
            stylesheet: ".a::placeholder, .b::selection { color: var(--color-text); }",
            message: "a rule of a pseudo-element is shared with other selectors",
            column: 1,
        },
        Rejection {
            stylesheet: ".a { caret-color: #fff; }",
            message: "raw colors are not supported",
            column: 19,
        },
        Rejection {
            stylesheet: ".a { caret-color: var(--space-2); }",
            message: "`--space-2` is a length, not a color",
            column: 19,
        },
    ]);
}
