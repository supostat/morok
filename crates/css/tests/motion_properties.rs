mod support;

use morok_css::TokenId;
use morok_style::{
    AnimatableProperties, AnimatableProperty, AnimationDirection, AnimationName, BoxShadow, Corner, Declaration,
    Easing, EasingValue, FillMode, IterationCount, Keyframe, Keyframes, Milliseconds, Side, StepPosition, TimeValue,
};
use support::{STYLESHEET_PATH, base_tokens, declarations, px, rejected, sized, style_sheet};

const EASE: EasingValue = EasingValue::Curve(Easing::CubicBezier {
    x1: 0.25,
    y1: 0.1,
    x2: 0.25,
    y2: 1.0,
});

fn milliseconds(milliseconds: f32) -> TimeValue {
    TimeValue::Milliseconds(Milliseconds(milliseconds))
}

fn one(property: AnimatableProperty) -> AnimatableProperties {
    AnimatableProperties::of(property)
}

fn duration_token(name: &str) -> TimeValue {
    match base_tokens().id(name) {
        Some(TokenId::Duration(token)) => TimeValue::Token(token),
        other => panic!("`--{name}` is not a duration token: {other:?}"),
    }
}

fn easing_token(name: &str) -> EasingValue {
    match base_tokens().id(name) {
        Some(TokenId::Easing(token)) => EasingValue::Token(token),
        other => panic!("`--{name}` is not an easing token: {other:?}"),
    }
}

/// The message and the column of the one diagnostic a stylesheet of one line produces.
fn rejection(stylesheet: &str) -> (String, usize) {
    let diagnostics = rejected(stylesheet);
    assert_eq!(diagnostics.len(), 1, "{stylesheet}: {diagnostics:?}");
    let diagnostic = &diagnostics[0];
    assert_eq!(
        (diagnostic.path.to_str(), diagnostic.line),
        (Some(STYLESHEET_PATH), 1),
        "{stylesheet}"
    );
    (diagnostic.message.clone(), diagnostic.column)
}

#[test]
fn a_transition_compiles_into_its_four_longhands_one_entry_per_transition() {
    assert_eq!(
        declarations("transition: opacity 200ms, background-color 0.3s var(--motion-ease) -50ms;"),
        [
            Declaration::TransitionProperty(
                vec![
                    one(AnimatableProperty::Opacity),
                    one(AnimatableProperty::BackgroundColor)
                ]
                .into()
            ),
            Declaration::TransitionDuration(vec![milliseconds(200.0), milliseconds(300.0)].into()),
            Declaration::TransitionTimingFunction(vec![EASE, easing_token("motion-ease")].into()),
            Declaration::TransitionDelay(vec![milliseconds(0.0), milliseconds(-50.0)].into()),
        ]
    );
}

#[test]
fn the_parts_of_a_transition_come_in_any_order_and_take_motion_tokens() {
    assert_eq!(
        declarations("transition: var(--motion-fast) linear transform var(--motion-base);"),
        [
            Declaration::TransitionProperty(vec![one(AnimatableProperty::Transform)].into()),
            Declaration::TransitionDuration(vec![duration_token("motion-fast")].into()),
            Declaration::TransitionTimingFunction(vec![EasingValue::Curve(Easing::Linear)].into()),
            Declaration::TransitionDelay(vec![duration_token("motion-base")].into()),
        ]
    );
    assert_eq!(
        declarations("transition: color 1s steps(4, jump-start), width 1s cubic-bezier(0.3, -0.5, 0.7, 1.5);")[2],
        Declaration::TransitionTimingFunction(
            vec![
                EasingValue::Curve(Easing::Steps {
                    count: 4,
                    position: StepPosition::JumpStart,
                }),
                EasingValue::Curve(Easing::CubicBezier {
                    x1: 0.3,
                    y1: -0.5,
                    x2: 0.7,
                    y2: 1.5,
                }),
            ]
            .into()
        )
    );
}

#[test]
fn an_omitted_part_takes_its_initial_value_and_a_bare_duration_transitions_all() {
    assert_eq!(
        declarations("transition: 150ms;"),
        [
            Declaration::TransitionProperty(vec![AnimatableProperties::ALL].into()),
            Declaration::TransitionDuration(vec![milliseconds(150.0)].into()),
            Declaration::TransitionTimingFunction(vec![EASE].into()),
            Declaration::TransitionDelay(vec![milliseconds(0.0)].into()),
        ]
    );
    assert_eq!(
        declarations("transition: none;")[0],
        Declaration::TransitionProperty(vec![AnimatableProperties::NONE].into())
    );
}

#[test]
fn a_shorthand_name_transitions_every_longhand_it_sets() {
    let properties = |name: &str| match &declarations(&format!("transition: {name} 1s;"))[0] {
        Declaration::TransitionProperty(properties) => properties[0].iter().collect::<Vec<_>>(),
        other => panic!("{other:?}"),
    };

    assert_eq!(properties("padding"), Side::ALL.map(AnimatableProperty::Padding));
    assert_eq!(
        properties("border-radius"),
        Corner::ALL.map(AnimatableProperty::BorderRadius)
    );
    assert_eq!(
        properties("border-left"),
        [
            AnimatableProperty::BorderWidth(Side::Left),
            AnimatableProperty::BorderColor(Side::Left)
        ]
    );
    assert_eq!(properties("background"), [AnimatableProperty::BackgroundColor]);
    assert_eq!(
        properties("gap"),
        [AnimatableProperty::RowGap, AnimatableProperty::ColumnGap]
    );
    assert_eq!(
        properties("outline"),
        [AnimatableProperty::OutlineWidth, AnimatableProperty::OutlineColor]
    );
}

#[test]
fn a_transition_of_what_cannot_be_interpolated_is_a_diagnostic() {
    let cases = [
        (".a { transition: display 1s; }", "`display` cannot be transitioned", 18),
        (
            ".a { transition: frobnicate 1s; }",
            "`frobnicate` is not a property",
            18,
        ),
        (".a { transition: opacity -1s; }", "a duration cannot be negative", 26),
        (
            ".a { transition: opacity var(--color-bg); }",
            "`--color-bg` is a color, not a duration or an easing",
            26,
        ),
        (
            ".a { transition: opacity 1s, none 2s; }",
            "`transition` does not support this value",
            18,
        ),
        (
            ".a { transition: opacity 1s 2s 3s; }",
            "`transition` does not support this value",
            18,
        ),
        (
            ".a { transition: opacity width 1s; }",
            "`transition` does not support this value",
            18,
        ),
        (".a { transition: opacity 1s ease; }", "`ease` is not a property", 29),
    ];
    for (stylesheet, message, column) in cases {
        assert_eq!(rejection(stylesheet), (message.to_owned(), column), "{stylesheet}");
    }
}

#[test]
fn an_animation_compiles_into_its_seven_longhands() {
    let style_sheet = style_sheet(
        "@keyframes pulse { to { opacity: 0.5; } }\n\
         .a { animation: pulse 2s var(--motion-ease-out) 100ms infinite alternate both, none; }",
    );

    assert_eq!(
        style_sheet.rules[0].declarations.to_vec(),
        [
            Declaration::AnimationName(vec![AnimationName::Keyframes(Keyframes::new(0)), AnimationName::None].into()),
            Declaration::AnimationDuration(vec![milliseconds(2000.0), milliseconds(0.0)].into()),
            Declaration::AnimationTimingFunction(vec![easing_token("motion-ease-out"), EASE].into()),
            Declaration::AnimationDelay(vec![milliseconds(100.0), milliseconds(0.0)].into()),
            Declaration::AnimationIterationCount(vec![IterationCount::Infinite, IterationCount::Count(1.0)].into()),
            Declaration::AnimationDirection(vec![AnimationDirection::Alternate, AnimationDirection::Normal].into()),
            Declaration::AnimationFillMode(vec![FillMode::Both, FillMode::None].into()),
        ]
    );
}

#[test]
fn a_keyword_of_an_animation_goes_to_the_first_part_that_takes_it() {
    let animation = |value: &str| {
        let style_sheet = style_sheet(&format!(
            "@keyframes none-like {{ to {{ opacity: 0; }} }}\n.a {{ animation: {value}; }}"
        ));
        style_sheet.rules[0].declarations.to_vec()
    };

    let filled_first = animation("forwards none 1s");
    assert_eq!(
        filled_first[0],
        Declaration::AnimationName(vec![AnimationName::None].into())
    );
    assert_eq!(
        filled_first[6],
        Declaration::AnimationFillMode(vec![FillMode::Forwards].into())
    );
    let named_after = animation("none none-like 1s");
    assert_eq!(
        named_after[0],
        Declaration::AnimationName(vec![AnimationName::Keyframes(Keyframes::new(0))].into())
    );
    assert_eq!(
        named_after[6],
        Declaration::AnimationFillMode(vec![FillMode::None].into())
    );
    let reversed = animation("reverse none-like 1.5 1s");
    assert_eq!(
        reversed[0],
        Declaration::AnimationName(vec![AnimationName::Keyframes(Keyframes::new(0))].into())
    );
    assert_eq!(
        reversed[4],
        Declaration::AnimationIterationCount(vec![IterationCount::Count(1.5)].into())
    );
    assert_eq!(
        reversed[5],
        Declaration::AnimationDirection(vec![AnimationDirection::Reverse].into())
    );
}

#[test]
fn an_animation_names_keyframes_of_any_stylesheet_compiled_with_it_declared_before_or_after() {
    let style_sheet = style_sheet(
        ".a { animation: fade 1s; }\n@keyframes spin { to { transform: rotate(360deg); } }\n\
         @keyframes fade { from { opacity: 0; } }\n.b { animation: spin 1s; }",
    );

    assert_eq!(style_sheet.keyframes[0].name, "spin");
    assert_eq!(style_sheet.keyframes[1].name, "fade");
    assert_eq!(
        style_sheet.rules[0].declarations[0],
        Declaration::AnimationName(vec![AnimationName::Keyframes(Keyframes::new(1))].into())
    );
    assert_eq!(
        style_sheet.rules[1].declarations[0],
        Declaration::AnimationName(vec![AnimationName::Keyframes(Keyframes::new(0))].into())
    );
}

#[test]
fn keyframes_are_ordered_by_offset_with_the_blocks_of_one_offset_merged_in_source_order() {
    let style_sheet = style_sheet(
        "@keyframes grow {\n\
           to { width: 40px; }\n\
           from, 50% { width: 10px; opacity: 0; }\n\
           50% { width: 30px; }\n\
         }",
    );

    assert_eq!(
        style_sheet.keyframes[0].keyframes.to_vec(),
        [
            Keyframe {
                offset: 0.0,
                declarations: vec![Declaration::Width(sized(px(10.0))), Declaration::Opacity(0.0)].into(),
            },
            Keyframe {
                offset: 0.5,
                declarations: vec![
                    Declaration::Width(sized(px(10.0))),
                    Declaration::Opacity(0.0),
                    Declaration::Width(sized(px(30.0))),
                ]
                .into(),
            },
            Keyframe {
                offset: 1.0,
                declarations: vec![Declaration::Width(sized(px(40.0)))].into(),
            },
        ]
    );
}

#[test]
fn a_misformed_keyframes_rule_or_animation_is_a_diagnostic() {
    let cases = [
        (
            ".a { animation: missing 1s; }",
            "no `@keyframes` rule is named `missing`",
            17,
        ),
        (
            "@keyframes a { to { opacity: 0; } } .b { animation: a 1s paused; }",
            "`paused` is not supported",
            58,
        ),
        (
            "@keyframes a { to { opacity: 0; } } @keyframes a { to { opacity: 1; } }",
            "`@keyframes a` is declared twice",
            48,
        ),
        (
            "@keyframes none { to { opacity: 0; } }",
            "`none` cannot name keyframes",
            12,
        ),
        (
            "@keyframes \"quoted\" { to { opacity: 0; } }",
            "`\"quoted\"` is not a name of keyframes",
            12,
        ),
        (
            "@keyframes a { 120% { opacity: 0; } }",
            "`120%` is not a keyframe selector",
            16,
        ),
        (
            "@keyframes a { from, middle { opacity: 0; } }",
            "`middle` is not a keyframe selector",
            22,
        ),
        (
            "@keyframes a { to { display: none; } }",
            "`display` cannot be animated",
            21,
        ),
        (
            "@keyframes a { to { border: 1px solid var(--color-line); } }",
            "`border` cannot be animated",
            21,
        ),
        (
            "@keyframes a { to { transition: opacity 1s; } }",
            "`transition` cannot be animated",
            21,
        ),
        (
            "@media (min-width: 1px) { .a { opacity: 0; } }",
            "`@media` is not supported",
            1,
        ),
    ];
    for (stylesheet, message, column) in cases {
        assert_eq!(rejection(stylesheet), (message.to_owned(), column), "{stylesheet}");
    }
}

#[test]
fn a_keyframe_takes_every_animatable_property_with_its_tokens() {
    let style_sheet = style_sheet(
        "@keyframes all { 50% { color: var(--color-text); padding: 0 var(--space-2); box-shadow: var(--shadow-float); \
         transform: translateX(10px); border-color: var(--color-line); border-width: 2px; } }",
    );

    let properties: Vec<_> = style_sheet.keyframes[0].keyframes[0]
        .declarations
        .iter()
        .map(Declaration::animatable_property)
        .collect();
    assert!(properties.iter().all(Option::is_some), "{properties:?}");
    assert_eq!(properties.len(), 1 + 4 + 1 + 1 + 4 + 4);
    assert!(matches!(
        style_sheet.keyframes[0].keyframes[0].declarations[5],
        Declaration::BoxShadow(BoxShadow::Token(_))
    ));
}
