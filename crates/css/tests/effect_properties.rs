mod support;

use morok_style::{BackdropFilterFunction, BlendMode, Declaration, FilterFunction, TransformFunction};
use support::{STYLESHEET_PATH, declarations, fraction, px, px_length, rejected, token, token_length};

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
fn an_opacity_is_a_number_or_a_percentage_clamped_to_one() {
    assert_eq!(declarations("opacity: 0.5;"), [Declaration::Opacity(0.5)]);
    assert_eq!(declarations("opacity: 40%;"), [Declaration::Opacity(0.4)]);
    assert_eq!(declarations("opacity: 3;"), [Declaration::Opacity(1.0)]);
}

#[test]
fn a_transform_is_a_list_of_functions_applied_in_order() {
    assert_eq!(
        declarations("transform: translate(10px, 25%) scale(2) rotate(45deg);"),
        [Declaration::Transform(
            vec![
                TransformFunction::Translate(px(10.0), fraction(0.25)),
                TransformFunction::Scale(2.0, 2.0),
                TransformFunction::Rotate(45.0),
            ]
            .into()
        )]
    );
    assert_eq!(
        declarations("transform: translateX(var(--space-4)) translateY(-50%) scaleX(0.5) scaleY(1.5) scale(1, 2);"),
        [Declaration::Transform(
            vec![
                TransformFunction::TranslateX(token("space-4")),
                TransformFunction::TranslateY(fraction(-0.5)),
                TransformFunction::ScaleX(0.5),
                TransformFunction::ScaleY(1.5),
                TransformFunction::Scale(1.0, 2.0),
            ]
            .into()
        )]
    );
    assert_eq!(
        declarations("transform: translate(8px);"),
        [Declaration::Transform(
            vec![TransformFunction::Translate(px(8.0), px(0.0))].into()
        )]
    );
    assert_eq!(
        declarations("transform: none;"),
        [Declaration::Transform(vec![].into())]
    );
}

#[test]
fn a_transform_origin_is_two_coordinates_or_one_with_the_other_at_the_center() {
    assert_eq!(
        declarations("transform-origin: left top;"),
        [Declaration::TransformOrigin(fraction(0.0), fraction(0.0))]
    );
    assert_eq!(
        declarations("transform-origin: bottom right;"),
        [Declaration::TransformOrigin(fraction(1.0), fraction(1.0))]
    );
    assert_eq!(
        declarations("transform-origin: 50% 20px;"),
        [Declaration::TransformOrigin(fraction(0.5), px(20.0))]
    );
    assert_eq!(
        declarations("transform-origin: center;"),
        [Declaration::TransformOrigin(fraction(0.5), fraction(0.5))]
    );
    assert_eq!(
        declarations("transform-origin: bottom;"),
        [Declaration::TransformOrigin(fraction(0.5), fraction(1.0))]
    );
    assert_eq!(
        declarations("transform-origin: var(--space-2);"),
        [Declaration::TransformOrigin(token("space-2"), fraction(0.5))]
    );
}

#[test]
fn a_filter_is_a_list_of_functions_with_amounts_as_numbers_or_percentages() {
    assert_eq!(
        declarations("filter: blur(4px) brightness(1.2) contrast(80%) grayscale(1) saturate(200%) opacity(50%);"),
        [Declaration::Filter(
            vec![
                FilterFunction::Blur(px_length(4.0)),
                FilterFunction::Brightness(1.2),
                FilterFunction::Contrast(0.8),
                FilterFunction::Grayscale(1.0),
                FilterFunction::Saturate(2.0),
                FilterFunction::Opacity(0.5),
            ]
            .into()
        )]
    );
    assert_eq!(
        declarations("filter: blur(var(--space-1)) grayscale(150%);"),
        [Declaration::Filter(
            vec![
                FilterFunction::Blur(token_length("space-1")),
                FilterFunction::Grayscale(1.0),
            ]
            .into()
        )]
    );
    assert_eq!(declarations("filter: none;"), [Declaration::Filter(vec![].into())]);
}

#[test]
fn a_backdrop_filter_is_a_list_of_its_own_functions() {
    assert_eq!(
        declarations("backdrop-filter: blur(12px) brightness(0.8) saturate(1.5);"),
        [Declaration::BackdropFilter(
            vec![
                BackdropFilterFunction::Blur(px_length(12.0)),
                BackdropFilterFunction::Brightness(0.8),
                BackdropFilterFunction::Saturate(1.5),
            ]
            .into()
        )]
    );
    assert_eq!(
        declarations("backdrop-filter: none;"),
        [Declaration::BackdropFilter(vec![].into())]
    );
}

#[test]
fn a_blend_mode_is_one_of_the_separable_modes() {
    for (keyword, mode) in [
        ("normal", BlendMode::Normal),
        ("multiply", BlendMode::Multiply),
        ("screen", BlendMode::Screen),
        ("overlay", BlendMode::Overlay),
        ("darken", BlendMode::Darken),
        ("lighten", BlendMode::Lighten),
    ] {
        assert_eq!(
            declarations(&format!("mix-blend-mode: {keyword};")),
            [Declaration::MixBlendMode(mode)]
        );
    }
}

#[test]
fn a_function_outside_the_list_of_its_property_is_an_error() {
    assert_rejected(&[
        Rejection {
            declaration: "transform: matrix(1, 0, 0, 1, 10, 20)",
            message: "`matrix()` is not supported in `transform`",
            column: 17,
        },
        Rejection {
            declaration: "transform: rotate3d(1, 0, 0, 45deg)",
            message: "`rotate3d()` is not supported in `transform`",
            column: 17,
        },
        Rejection {
            declaration: "transform: scale(2) skew(10deg)",
            message: "`skew()` is not supported in `transform`",
            column: 26,
        },
        Rejection {
            declaration: "filter: drop-shadow(0 0 4px var(--color-scrim))",
            message: "`drop-shadow()` is not supported in `filter`",
            column: 14,
        },
        Rejection {
            declaration: "filter: blur(2px) hue-rotate(90deg)",
            message: "`hue-rotate()` is not supported in `filter`",
            column: 24,
        },
        Rejection {
            declaration: "backdrop-filter: contrast(2)",
            message: "`contrast()` is not supported in `backdrop-filter`",
            column: 23,
        },
        Rejection {
            declaration: "backdrop-filter: grayscale(1)",
            message: "`grayscale()` is not supported in `backdrop-filter`",
            column: 23,
        },
    ]);
}

#[test]
fn an_angle_outside_degrees_and_a_value_outside_the_grammar_are_errors() {
    assert_rejected(&[
        Rejection {
            declaration: "transform: rotate(1rad)",
            message: "the unit `rad` is not supported",
            column: 24,
        },
        Rejection {
            declaration: "opacity: -0.5",
            message: "`opacity` does not support this value",
            column: 15,
        },
        Rejection {
            declaration: "filter: blur(-4px)",
            message: "`filter` does not support this value",
            column: 14,
        },
        Rejection {
            declaration: "mix-blend-mode: hue",
            message: "`mix-blend-mode` does not support this value",
            column: 22,
        },
        Rejection {
            declaration: "transform-origin: 10px 20px 30px",
            message: "`transform-origin` does not support this value",
            column: 24,
        },
    ]);
}
