mod support;

use morok_css::TokenId;
use morok_style::{
    Declaration, FontFamilies, FontStyle, FontVariantNumeric, FontWeight, FontWeightValue, Leading, Length, LineHeight,
    Pixels, TextAlign, TextDecoration, TextOverflow, Token, TokenTerm, WhiteSpace,
};
use support::{STYLESHEET_PATH, base_tokens, declarations, length_token, px_length, rejected, token_length};

fn font_families_token(name: &str) -> Token<FontFamilies> {
    match base_tokens().id(name) {
        Some(TokenId::FontFamilies(token)) => token,
        other => panic!("`--{name}` is not a font families token: {other:?}"),
    }
}

fn font_weight_token(name: &str) -> Token<FontWeight> {
    match base_tokens().id(name) {
        Some(TokenId::FontWeight(token)) => token,
        other => panic!("`--{name}` is not a font weight token: {other:?}"),
    }
}

fn leading_token(name: &str) -> Token<Leading> {
    match base_tokens().id(name) {
        Some(TokenId::Leading(token)) => token,
        other => panic!("`--{name}` is not a line height token: {other:?}"),
    }
}

/// A declaration with the one diagnostic it must produce: its message and its column on line 1.
struct Rejection {
    declaration: &'static str,
    message: &'static str,
    column: usize,
}

fn assert_rejected(rejections: &[Rejection]) {
    for rejection in rejections {
        let diagnostics = rejected(&format!(".a {{ {}; }}", rejection.declaration));

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
            rejection.declaration
        );
    }
}

#[test]
fn a_font_family_is_a_font_token() {
    assert_eq!(
        declarations("font-family: var(--font-mono);"),
        [Declaration::FontFamily(font_families_token("font-mono"))]
    );
}

#[test]
fn a_font_size_is_a_size_token_or_a_calculation_over_one() {
    assert_eq!(
        declarations("font-size: var(--text-lg);"),
        [Declaration::FontSize(token_length("text-lg"))]
    );
    assert_eq!(
        declarations("font-size: calc(var(--text-md) * 2 + 1px);"),
        [Declaration::FontSize(Length {
            pixels: Pixels(1.0),
            tokens: vec![TokenTerm {
                token: length_token("text-md"),
                factor: 2.0,
            }]
            .into(),
        })]
    );
}

#[test]
fn a_font_weight_is_a_weight_token_or_a_keyword() {
    assert_eq!(
        declarations("font-weight: var(--text-weight-semibold);"),
        [Declaration::FontWeight(FontWeightValue::Token(font_weight_token(
            "text-weight-semibold"
        )))]
    );
    assert_eq!(
        declarations("font-weight: normal;"),
        [Declaration::FontWeight(FontWeightValue::Keyword(FontWeight(400)))]
    );
    assert_eq!(
        declarations("font-weight: bold;"),
        [Declaration::FontWeight(FontWeightValue::Keyword(FontWeight(700)))]
    );
}

#[test]
fn a_line_height_is_normal_or_a_leading_token() {
    assert_eq!(
        declarations("line-height: normal;"),
        [Declaration::LineHeight(LineHeight::Normal)]
    );
    assert_eq!(
        declarations("line-height: var(--text-leading-tight);"),
        [Declaration::LineHeight(LineHeight::Token(leading_token(
            "text-leading-tight"
        )))]
    );
}

#[test]
fn a_letter_spacing_is_a_length_of_either_sign_and_normal_is_none() {
    assert_eq!(
        declarations("letter-spacing: 0.5px;"),
        [Declaration::LetterSpacing(px_length(0.5))]
    );
    assert_eq!(
        declarations("letter-spacing: -0.25px;"),
        [Declaration::LetterSpacing(px_length(-0.25))]
    );
    assert_eq!(
        declarations("letter-spacing: normal;"),
        [Declaration::LetterSpacing(px_length(0.0))]
    );
}

#[test]
fn every_keyword_of_a_text_property_compiles_into_its_value() {
    let keywords: [(&str, Declaration); 18] = [
        ("font-style: normal", Declaration::FontStyle(FontStyle::Normal)),
        ("font-style: italic", Declaration::FontStyle(FontStyle::Italic)),
        ("text-align: start", Declaration::TextAlign(TextAlign::Start)),
        ("text-align: end", Declaration::TextAlign(TextAlign::End)),
        ("text-align: left", Declaration::TextAlign(TextAlign::Left)),
        ("text-align: right", Declaration::TextAlign(TextAlign::Right)),
        ("text-align: center", Declaration::TextAlign(TextAlign::Center)),
        (
            "text-decoration: none",
            Declaration::TextDecoration(TextDecoration::None),
        ),
        (
            "text-decoration: underline",
            Declaration::TextDecoration(TextDecoration::Underline),
        ),
        (
            "text-decoration: line-through",
            Declaration::TextDecoration(TextDecoration::LineThrough),
        ),
        ("text-overflow: clip", Declaration::TextOverflow(TextOverflow::Clip)),
        (
            "text-overflow: ellipsis",
            Declaration::TextOverflow(TextOverflow::Ellipsis),
        ),
        ("white-space: normal", Declaration::WhiteSpace(WhiteSpace::Normal)),
        ("white-space: nowrap", Declaration::WhiteSpace(WhiteSpace::Nowrap)),
        ("white-space: pre", Declaration::WhiteSpace(WhiteSpace::Pre)),
        ("white-space: pre-wrap", Declaration::WhiteSpace(WhiteSpace::PreWrap)),
        (
            "font-variant-numeric: normal",
            Declaration::FontVariantNumeric(FontVariantNumeric::Normal),
        ),
        (
            "font-variant-numeric: tabular-nums",
            Declaration::FontVariantNumeric(FontVariantNumeric::TabularNums),
        ),
    ];

    for (declaration, compiled) in keywords {
        assert_eq!(declarations(&format!("{declaration};")), [compiled], "{declaration}");
    }
}

#[test]
fn typography_written_out_in_a_stylesheet_is_an_error() {
    assert_rejected(&[
        Rejection {
            declaration: "font-family: \"SF Pro Text\", sans-serif",
            message: "raw font families are not supported",
            column: 19,
        },
        Rejection {
            declaration: "font-family: system-ui",
            message: "raw font families are not supported",
            column: 19,
        },
        Rejection {
            declaration: "font-size: 13px",
            message: "raw font sizes are not supported",
            column: 17,
        },
        Rejection {
            declaration: "font-weight: 600",
            message: "raw font weights are not supported",
            column: 19,
        },
        Rejection {
            declaration: "line-height: 1.4",
            message: "raw line heights are not supported",
            column: 19,
        },
        Rejection {
            declaration: "line-height: 20px",
            message: "raw line heights are not supported",
            column: 19,
        },
    ]);
}

#[test]
fn a_raw_value_is_reported_with_the_token_family_it_comes_from() {
    let diagnostics = rejected(".title { font-weight: 600; }");

    assert_eq!(
        diagnostics[0].to_string(),
        "error[morok-css]: raw font weights are not supported\n --> component.css:1:23\n  |\n1 | .title { font-weight: 600; }\n  |                       ^^^ it comes from a `text-weight-` token: `var(--text-weight-…)`"
    );
}

#[test]
fn a_token_of_another_kind_is_an_error() {
    assert_rejected(&[
        Rejection {
            declaration: "font-family: var(--color-bg)",
            message: "`--color-bg` is a color, not a list of font families",
            column: 19,
        },
        Rejection {
            declaration: "font-size: var(--text-weight-medium)",
            message: "`--text-weight-medium` is a font weight, not a length",
            column: 17,
        },
        Rejection {
            declaration: "font-weight: var(--text-leading)",
            message: "`--text-leading` is a line height, not a font weight",
            column: 19,
        },
        Rejection {
            declaration: "line-height: var(--text-md)",
            message: "`--text-md` is a length, not a line height",
            column: 19,
        },
    ]);
}

#[test]
fn units_and_percentages_outside_the_subset_are_errors() {
    assert_rejected(&[
        Rejection {
            declaration: "letter-spacing: 0.02em",
            message: "the unit `em` is not supported",
            column: 22,
        },
        Rejection {
            declaration: "letter-spacing: 2%",
            message: "a percentage is not supported here",
            column: 22,
        },
        Rejection {
            declaration: "font-size: calc(var(--text-md) + 10%)",
            message: "a percentage is not supported here",
            column: 17,
        },
    ]);
}

#[test]
fn a_keyword_outside_the_subset_is_an_error() {
    assert_rejected(&[
        Rejection {
            declaration: "font-style: oblique",
            message: "`font-style` does not support this value",
            column: 18,
        },
        Rejection {
            declaration: "text-align: justify",
            message: "`text-align` does not support this value",
            column: 18,
        },
        Rejection {
            declaration: "text-decoration: overline",
            message: "`text-decoration` does not support this value",
            column: 23,
        },
        Rejection {
            declaration: "text-decoration: underline dotted",
            message: "`text-decoration` does not support this value",
            column: 23,
        },
        Rejection {
            declaration: "text-overflow: fade",
            message: "`text-overflow` does not support this value",
            column: 21,
        },
        Rejection {
            declaration: "white-space: pre-line",
            message: "`white-space` does not support this value",
            column: 19,
        },
        Rejection {
            declaration: "font-variant-numeric: oldstyle-nums",
            message: "`font-variant-numeric` does not support this value",
            column: 28,
        },
        Rejection {
            declaration: "font-weight: bolder",
            message: "`font-weight` does not support this value",
            column: 19,
        },
        Rejection {
            declaration: "line-height: inherit",
            message: "`line-height` does not support this value",
            column: 19,
        },
    ]);
}
