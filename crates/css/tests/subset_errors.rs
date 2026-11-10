mod support;

use morok_css::{SourceFile, compile_stylesheets};
use support::{STYLESHEET_PATH, base_tokens, rejected};

/// A stylesheet with the one diagnostic it must produce: its message and its column on line 1.
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
fn a_descendant_selector_is_an_error() {
    assert_rejected(&[Rejection {
        stylesheet: ".list .row { display: block; }",
        message: "descendant selectors are not supported",
        column: 1,
    }]);
}

#[test]
fn a_descendant_selector_is_reported_as_the_design_book_shows() {
    let stylesheet = format!("{}.list .row .title {{ font-weight: 600 }}\n", "\n".repeat(13));
    let source = SourceFile::new("components/list.css", stylesheet);

    let diagnostics = compile_stylesheets(&[source], &base_tokens()).expect_err("the stylesheet is rejected");

    assert_eq!(
        diagnostics.as_slice()[0].to_string(),
        "error[morok-css]: descendant selectors are not supported\n  --> components/list.css:14:1\n   |\n14 | .list .row .title { font-weight: 600 }\n   | ^^^^^^^^^^^^^^^^^ give the element its own class: .list-title"
    );
}

#[test]
fn a_second_child_step_is_an_error() {
    assert_rejected(&[Rejection {
        stylesheet: ".a > .b > .c { display: block; }",
        message: "only one child step is supported",
        column: 1,
    }]);
}

#[test]
fn structural_and_attribute_selectors_are_errors() {
    assert_rejected(&[
        Rejection {
            stylesheet: ".row:nth-child(2) { display: block; }",
            message: "`:nth-child()` is not supported",
            column: 5,
        },
        Rejection {
            stylesheet: ".row:has(.cell) { display: block; }",
            message: "`:has()` is not supported",
            column: 5,
        },
        Rejection {
            stylesheet: "* { display: block; }",
            message: "the universal selector `*` is not supported",
            column: 1,
        },
        Rejection {
            stylesheet: ".row[data-kind] { display: block; }",
            message: "attribute selectors are not supported",
            column: 5,
        },
    ]);
}

#[test]
fn an_unknown_pseudo_class_or_pseudo_element_is_an_error() {
    assert_rejected(&[
        Rejection {
            stylesheet: ".a:visited { display: block; }",
            message: "the pseudo-class `:visited` is not supported",
            column: 3,
        },
        Rejection {
            stylesheet: ".a::before { display: block; }",
            message: "the pseudo-element `::before` is not supported",
            column: 3,
        },
    ]);
}

#[test]
fn selectors_outside_classes_states_and_one_child_step_are_errors() {
    assert_rejected(&[
        Rejection {
            stylesheet: "div { display: block; }",
            message: "type selectors are not supported: `div`",
            column: 1,
        },
        Rejection {
            stylesheet: "#main { display: block; }",
            message: "ID selectors are not supported",
            column: 1,
        },
        Rejection {
            stylesheet: ".a + .b { display: block; }",
            message: "sibling combinators are not supported",
            column: 4,
        },
        Rejection {
            stylesheet: ":hover { display: block; }",
            message: "a state needs a class to belong to",
            column: 1,
        },
        Rejection {
            stylesheet: ".Button { display: block; }",
            message: "`.Button` is not a lowercase kebab-case class name",
            column: 1,
        },
        Rejection {
            stylesheet: ".self { display: block; }",
            message: "`.self` cannot name a class",
            column: 1,
        },
        Rejection {
            stylesheet: ".a > { display: block; }",
            message: "a selector cannot end with `>`",
            column: 5,
        },
    ]);
}

#[test]
fn a_raw_color_is_an_error() {
    assert_rejected(&[
        Rejection {
            stylesheet: ".a { color: #fff; }",
            message: "raw colors are not supported",
            column: 13,
        },
        Rejection {
            stylesheet: ".a { background-color: rgb(0, 0, 0); }",
            message: "raw colors are not supported",
            column: 24,
        },
        Rejection {
            stylesheet: ".a { border: 1px solid #ccc; }",
            message: "raw colors are not supported",
            column: 24,
        },
        Rejection {
            stylesheet: ".a { background: linear-gradient(#fff, transparent); }",
            message: "raw colors are not supported",
            column: 34,
        },
    ]);
}

#[test]
fn an_unknown_token_is_an_error() {
    assert_rejected(&[Rejection {
        stylesheet: ".a { color: var(--color-nope); }",
        message: "unknown token `--color-nope`",
        column: 13,
    }]);
}

#[test]
fn a_token_of_the_wrong_kind_for_the_property_is_an_error() {
    assert_rejected(&[
        Rejection {
            stylesheet: ".a { gap: var(--color-bg); }",
            message: "`--color-bg` is a color, not a length",
            column: 11,
        },
        Rejection {
            stylesheet: ".a { color: var(--space-2); }",
            message: "`--space-2` is a length, not a color",
            column: 13,
        },
        Rejection {
            stylesheet: ".a { box-shadow: var(--radius-sm); }",
            message: "`--radius-sm` is a length, not a shadow",
            column: 18,
        },
        Rejection {
            stylesheet: ".a { width: calc(100% - var(--motion-fast)); }",
            message: "`--motion-fast` is a duration, not a length",
            column: 25,
        },
    ]);
}

#[test]
fn an_unknown_or_not_yet_rendered_property_is_an_error() {
    assert_rejected(&[
        Rejection {
            stylesheet: ".a { float: left; }",
            message: "`float` is not a supported property",
            column: 6,
        },
        Rejection {
            stylesheet: ".a { cursor: pointer; }",
            message: "`cursor` is not a supported property",
            column: 6,
        },
        Rejection {
            stylesheet: ".a { --own-gap: 4px; }",
            message: "`--own-gap` is declared in a component stylesheet",
            column: 6,
        },
    ]);
}

#[test]
fn important_is_an_error() {
    assert_rejected(&[Rejection {
        stylesheet: ".a { color: var(--color-text) !important; }",
        message: "`!important` is not supported",
        column: 31,
    }]);
}

#[test]
fn an_at_rule_is_an_error() {
    assert_rejected(&[
        Rejection {
            stylesheet: "@media (min-width: 100px) { .a { display: none; } }",
            message: "`@media` is not supported",
            column: 1,
        },
        Rejection {
            stylesheet: "@import \"other.css\";",
            message: "`@import` is not supported",
            column: 1,
        },
        Rejection {
            stylesheet: "@font-face { font-family: Mono; }",
            message: "`@font-face` is not supported",
            column: 1,
        },
    ]);
}

#[test]
fn a_unit_outside_px_and_percent_is_an_error() {
    assert_rejected(&[
        Rejection {
            stylesheet: ".a { padding: 1em; }",
            message: "the unit `em` is not supported",
            column: 15,
        },
        Rejection {
            stylesheet: ".a { width: 2rem; }",
            message: "the unit `rem` is not supported",
            column: 13,
        },
        Rejection {
            stylesheet: ".a { height: calc(100vh - 4px); }",
            message: "the unit `vh` is not supported",
            column: 19,
        },
        Rejection {
            stylesheet: ".a { margin: 0 5vw; }",
            message: "the unit `vw` is not supported",
            column: 16,
        },
        Rejection {
            stylesheet: ".a { background-image: linear-gradient(0.25turn, var(--color-bg), transparent); }",
            message: "the unit `turn` is not supported",
            column: 40,
        },
    ]);
}

#[test]
fn a_value_outside_the_grammar_of_its_property_is_an_error() {
    assert_rejected(&[
        Rejection {
            stylesheet: ".a { display: inline-flex; }",
            message: "`display` does not support this value",
            column: 15,
        },
        Rejection {
            stylesheet: ".a { position: fixed; }",
            message: "`position` does not support this value",
            column: 16,
        },
        Rejection {
            stylesheet: ".a { padding: -4px; }",
            message: "`padding` does not support this value",
            column: 15,
        },
        Rejection {
            stylesheet: ".a { width: 4; }",
            message: "`width` does not support this value",
            column: 13,
        },
        Rejection {
            stylesheet: ".a { border-style: dashed; }",
            message: "`border-style` does not support this value",
            column: 20,
        },
        Rejection {
            stylesheet: ".a { grid-template-columns: minmax(1fr, 2fr); }",
            message: "`grid-template-columns` does not support this value",
            column: 29,
        },
        Rejection {
            stylesheet: ".a { width: calc(4px * 4px); }",
            message: "`width` does not support this value",
            column: 13,
        },
        Rejection {
            stylesheet: ".a { color: red; }",
            message: "`color` does not support this value",
            column: 13,
        },
        Rejection {
            stylesheet: ".a { width: 1e999px; }",
            message: "`width` does not support this value",
            column: 13,
        },
        Rejection {
            stylesheet: ".a { width: calc(1e999px + 1px); }",
            message: "`width` does not support this value",
            column: 13,
        },
    ]);
}

#[test]
fn css_the_renderer_does_not_draw_is_an_error() {
    assert_rejected(&[
        Rejection {
            stylesheet: ".a { border-radius: 50%; }",
            message: "a percentage is not supported here",
            column: 21,
        },
        Rejection {
            stylesheet: ".a { border-radius: 4px / 8px; }",
            message: "elliptical corners are not supported",
            column: 25,
        },
        Rejection {
            stylesheet: ".a { background-image: url(pattern.png); }",
            message: "`url()` is not supported",
            column: 24,
        },
        Rejection {
            stylesheet: ".a { background: var(--color-bg), var(--color-surface); }",
            message: "several background layers are not supported",
            column: 33,
        },
        Rejection {
            stylesheet: ".a { background-image: conic-gradient(var(--color-bg), transparent); }",
            message: "`conic-gradient()` is not supported",
            column: 24,
        },
        Rejection {
            stylesheet: ".a { background-image: radial-gradient(circle 40px, var(--color-bg), transparent); }",
            message: "an explicit radial gradient size is not supported",
            column: 47,
        },
        Rejection {
            stylesheet: ".a { box-shadow: 0 1px 2px var(--color-line); }",
            message: "raw shadows are not supported",
            column: 18,
        },
        Rejection {
            stylesheet: ".a { grid-template-columns: [start] 1fr; }",
            message: "named grid lines are not supported",
            column: 29,
        },
        Rejection {
            stylesheet: ".a { grid-column-start: sidebar; }",
            message: "named grid lines are not supported",
            column: 25,
        },
        Rejection {
            stylesheet: ".a { gap: var(--space-2, 4px); }",
            message: "a `var()` fallback is not supported",
            column: 11,
        },
    ]);
}

#[test]
fn a_malformed_rule_is_an_error() {
    assert_rejected(&[
        Rejection {
            stylesheet: ".a { display; }",
            message: "`display` has no value",
            column: 6,
        },
        Rejection {
            stylesheet: ".a { .b { display: block; } }",
            message: "expected a declaration",
            column: 6,
        },
        Rejection {
            stylesheet: ".a;",
            message: "expected a rule",
            column: 1,
        },
    ]);
}

#[test]
fn a_mismatch_names_what_the_property_takes() {
    let diagnostics = rejected(".a { display: inline-flex; }");

    assert_eq!(
        diagnostics[0].to_string(),
        "error[morok-css]: `display` does not support this value\n --> component.css:1:15\n  |\n1 | .a { display: inline-flex; }\n  |               ^^^^^^^^^^^ it takes `block`, `flex`, `grid` or `none`"
    );
}

#[test]
fn every_error_of_a_stylesheet_is_reported_in_source_order() {
    let diagnostics = rejected(".a .b { float: left; }\n.c { color: #fff; gap: 1em; }\n");

    let reported: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.message.as_str(), diagnostic.line, diagnostic.column))
        .collect();
    assert_eq!(
        reported,
        [
            ("descendant selectors are not supported", 1, 1),
            ("`float` is not a supported property", 1, 9),
            ("raw colors are not supported", 2, 13),
            ("the unit `em` is not supported", 2, 24),
        ]
    );
}

#[test]
fn errors_of_several_stylesheets_are_ordered_by_file() {
    let stylesheets = [
        SourceFile::new("b.css", ".b { float: left; }"),
        SourceFile::new("a.css", ".a { float: left; }"),
    ];

    let diagnostics = compile_stylesheets(&stylesheets, &base_tokens()).expect_err("the stylesheets are rejected");

    let paths: Vec<_> = diagnostics
        .as_slice()
        .iter()
        .map(|diagnostic| diagnostic.path.to_str())
        .collect();
    assert_eq!(paths, [Some("a.css"), Some("b.css")]);
}
