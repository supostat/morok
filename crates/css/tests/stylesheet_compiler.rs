mod support;

use std::borrow::Cow;

use morok_css::{SourceFile, compile_stylesheets};
use morok_style::{
    BackgroundImage, BoxShadow, Class, ColorStop, ColorValue, CompoundSelector, ContentAlignment, Corner, Declaration,
    Display, FlexDirection, FlexWrap, Gradient, GradientDirection, GridAutoFlow, GridLine, ItemAlignment,
    LengthPercentage, LengthPercentageAuto, LineStyle, LinearGradient, Overflow, Pixels, Position, RadialExtent,
    RadialGradient, RadialShape, RepeatCount, SelfAlignment, Side, States, TokenTerm, TrackBreadth, TrackListItem,
    TrackSize,
};
use support::{
    base_tokens, color, declarations, fraction, length_token, px, px_length, shadow_token, sized, style_sheet, token,
    token_length,
};

fn class(index: u32) -> Class {
    Class::new(index)
}

fn compound(classes: &[u32], states: States) -> CompoundSelector {
    CompoundSelector {
        classes: classes.iter().copied().map(Class::new).collect(),
        states,
    }
}

#[test]
fn a_class_rule_holds_its_declarations_and_is_keyed_by_its_class() {
    let style_sheet = style_sheet(".button { display: flex; }");

    assert_eq!(style_sheet.rules.len(), 1);
    assert_eq!(style_sheet.rules[0].selector.subject, compound(&[0], States::NONE));
    assert_eq!(style_sheet.rules[0].selector.parent, None);
    assert_eq!(
        *style_sheet.rules[0].declarations,
        [Declaration::Display(Display::Flex)]
    );
    assert_eq!(style_sheet.class_named("button"), Some(class(0)));
    assert_eq!(*style_sheet.class_style(class(0)).rules, [0]);
}

#[test]
fn compound_classes_and_states_form_the_subject() {
    let style_sheet = style_sheet(".button.primary:hover:focus-visible { display: none; }");

    assert_eq!(
        style_sheet.rules[0].selector.subject,
        compound(&[0, 1], States::HOVER.union(States::FOCUS_VISIBLE))
    );
    assert_eq!(*style_sheet.class_style(class(0)).rules, [0]);
    assert!(style_sheet.class_style(class(1)).rules.is_empty());
}

#[test]
fn every_state_pseudo_class_is_a_state() {
    let style_sheet =
        style_sheet(".field:hover:active:focus:focus-visible:focus-within:disabled:checked { display: none; }");

    let every_state = States::PSEUDO_CLASSES
        .into_iter()
        .fold(States::NONE, |states, (_, state)| states.union(state));
    assert_eq!(style_sheet.rules[0].selector.subject.states, every_state);
}

#[test]
fn a_child_step_names_the_parent() {
    let style_sheet = style_sheet(".table:hover > .tr.selected { display: none; }");

    let selector = &style_sheet.rules[0].selector;
    assert_eq!(selector.parent, Some(compound(&[0], States::HOVER)));
    assert_eq!(selector.subject, compound(&[1, 2], States::NONE));
    assert!(style_sheet.class_style(class(0)).rules.is_empty());
    assert_eq!(*style_sheet.class_style(class(1)).rules, [0]);
}

#[test]
fn a_comma_list_gives_every_selector_the_declarations() {
    let style_sheet = style_sheet(".primary:hover, .danger { display: none; }");

    assert_eq!(style_sheet.rules.len(), 2);
    assert_eq!(style_sheet.rules[0].selector.subject, compound(&[1], States::NONE));
    assert_eq!(style_sheet.rules[1].selector.subject, compound(&[0], States::HOVER));
    assert_eq!(style_sheet.rules[0].declarations, style_sheet.rules[1].declarations);
}

#[test]
fn rules_of_equal_specificity_keep_their_source_order() {
    let style_sheet = style_sheet(".button { display: flex; }\n.button { display: grid; }");

    assert_eq!(
        *style_sheet.rules[0].declarations,
        [Declaration::Display(Display::Flex)]
    );
    assert_eq!(
        *style_sheet.rules[1].declarations,
        [Declaration::Display(Display::Grid)]
    );
}

#[test]
fn a_more_specific_rule_comes_after_a_later_less_specific_one() {
    let style_sheet = style_sheet(
        ".button.primary { display: grid; }\n.button:hover { display: none; }\n.button { display: flex; }\n.card { display: block; }\n.toolbar > .button { display: flex; }",
    );

    let subjects: Vec<_> = style_sheet
        .rules
        .iter()
        .map(|rule| (rule.selector.subject.clone(), rule.selector.parent.is_some()))
        .collect();
    assert_eq!(
        subjects,
        [
            (compound(&[0], States::NONE), false),
            (compound(&[2], States::NONE), false),
            (compound(&[0, 1], States::NONE), false),
            (compound(&[0], States::HOVER), false),
            (compound(&[0], States::NONE), true),
        ]
    );
    assert_eq!(*style_sheet.class_style(class(0)).rules, [0, 2, 3, 4]);
    assert_eq!(*style_sheet.class_style(class(2)).rules, [1]);
}

#[test]
fn classes_get_their_ids_in_the_order_of_stylesheets_and_first_appearance() {
    let stylesheets = [
        SourceFile::new("a.css", ".toolbar > .button { display: flex; }"),
        SourceFile::new("b.css", ".card { display: block; }\n.button { display: grid; }"),
    ];

    let style_sheet = compile_stylesheets(&stylesheets, &base_tokens()).expect("the stylesheets compile");

    let names: Vec<_> = style_sheet.classes.iter().map(|class| class.name.as_ref()).collect();
    assert_eq!(names, ["toolbar", "button", "card"]);
    assert_eq!(style_sheet.class_named("card"), Some(class(2)));
    assert_eq!(style_sheet.class_named("absent"), None);
}

#[test]
fn a_class_named_as_a_rust_keyword_compiles() {
    let style_sheet = style_sheet(".box > .type { display: block; }");

    assert_eq!(style_sheet.class_named("box"), Some(class(0)));
    assert_eq!(style_sheet.class_named("type"), Some(class(1)));
}

#[test]
fn a_shorthand_of_four_sides_equals_its_longhands() {
    assert_eq!(
        declarations("padding: 0 var(--space-3);"),
        declarations("padding-top: 0; padding-right: var(--space-3); padding-bottom: 0; padding-left: var(--space-3);")
    );
    assert_eq!(
        declarations("margin: 1px;"),
        declarations("margin-top: 1px; margin-right: 1px; margin-bottom: 1px; margin-left: 1px;")
    );
    assert_eq!(
        declarations("margin: 1px auto 3px;"),
        declarations("margin-top: 1px; margin-right: auto; margin-bottom: 3px; margin-left: auto;")
    );
    assert_eq!(
        declarations("inset: 1px 2px 3px 4px;"),
        declarations("top: 1px; right: 2px; bottom: 3px; left: 4px;")
    );
    assert_eq!(
        declarations("border-width: 1px 2px;"),
        declarations(
            "border-top-width: 1px; border-right-width: 2px; border-bottom-width: 1px; border-left-width: 2px;"
        )
    );
    assert_eq!(
        declarations("border-radius: 1px 2px 3px;"),
        declarations(
            "border-top-left-radius: 1px; border-top-right-radius: 2px; border-bottom-right-radius: 3px; border-bottom-left-radius: 2px;"
        )
    );
}

#[test]
fn a_border_shorthand_equals_its_longhands_and_resets_what_it_omits() {
    assert_eq!(
        declarations("border-left: var(--color-line) solid 2px;"),
        declarations("border-left-width: 2px; border-left-style: solid; border-left-color: var(--color-line);")
    );
    assert_eq!(
        declarations("border: solid;"),
        [Side::Top, Side::Right, Side::Bottom, Side::Left]
            .into_iter()
            .flat_map(|side| [
                Declaration::BorderWidth(side, px_length(3.0)),
                Declaration::BorderStyle(side, LineStyle::Solid),
                Declaration::BorderColor(side, ColorValue::CurrentColor),
            ])
            .collect::<Vec<_>>()
    );
    assert_eq!(
        declarations("outline: 2px solid var(--color-focus);"),
        declarations("outline-width: 2px; outline-style: solid; outline-color: var(--color-focus);")
    );
    assert_eq!(
        declarations("border-style: solid none; border-color: transparent;"),
        declarations(
            "border-top-style: solid; border-right-style: none; border-bottom-style: solid; border-left-style: none; \
             border-top-color: transparent; border-right-color: transparent; border-bottom-color: transparent; border-left-color: transparent;"
        )
    );
}

#[test]
fn the_other_shorthands_equal_their_longhands() {
    assert_eq!(
        declarations("gap: 4px 10%;"),
        declarations("row-gap: 4px; column-gap: 10%;")
    );
    assert_eq!(
        declarations("gap: 4px;"),
        declarations("row-gap: 4px; column-gap: 4px;")
    );
    assert_eq!(
        declarations("overflow: hidden;"),
        declarations("overflow-x: hidden; overflow-y: hidden;")
    );
    assert_eq!(
        declarations("flex-flow: wrap column;"),
        declarations("flex-direction: column; flex-wrap: wrap;")
    );
    assert_eq!(
        declarations("grid-column: 2 / span 3;"),
        declarations("grid-column-start: 2; grid-column-end: span 3;")
    );
    assert_eq!(
        declarations("grid-row: -1;"),
        declarations("grid-row-start: -1; grid-row-end: auto;")
    );
    assert_eq!(
        declarations("background: var(--color-surface);"),
        declarations("background-color: var(--color-surface); background-image: none;")
    );
}

#[test]
fn the_flex_shorthand_fills_in_what_it_omits_as_css_does() {
    let flex = |grow, shrink, basis| {
        vec![
            Declaration::FlexGrow(grow),
            Declaration::FlexShrink(shrink),
            Declaration::FlexBasis(basis),
        ]
    };

    assert_eq!(declarations("flex: none;"), flex(0.0, 0.0, LengthPercentageAuto::Auto));
    assert_eq!(declarations("flex: auto;"), flex(1.0, 1.0, LengthPercentageAuto::Auto));
    assert_eq!(declarations("flex: 2;"), flex(2.0, 1.0, sized(fraction(0.0))));
    assert_eq!(declarations("flex: 2 3;"), flex(2.0, 3.0, sized(fraction(0.0))));
    assert_eq!(declarations("flex: 2 3 10px;"), flex(2.0, 3.0, sized(px(10.0))));
    assert_eq!(declarations("flex: 1 1 0;"), flex(1.0, 1.0, sized(px(0.0))));
    assert_eq!(declarations("flex: 50%;"), flex(1.0, 1.0, sized(fraction(0.5))));
    assert_eq!(declarations("flex: 10px 2;"), flex(2.0, 1.0, sized(px(10.0))));
}

#[test]
fn calc_folds_into_pixels_a_fraction_and_token_terms() {
    assert_eq!(
        declarations("width: calc(100% - var(--space-4) * 2 + 8px);"),
        [Declaration::Width(sized(LengthPercentage {
            pixels: Pixels(8.0),
            fraction: Some(1.0),
            tokens: Cow::Owned(vec![TokenTerm {
                token: length_token("space-4"),
                factor: -2.0,
            }]),
        }))]
    );
    assert_eq!(
        declarations("width: calc((4px + 12px) / 2 * (1 + 1));"),
        [Declaration::Width(sized(px(16.0)))]
    );
    assert_eq!(
        declarations("width: calc(calc(10px) + 50% - 50%);"),
        [Declaration::Width(sized(LengthPercentage {
            fraction: Some(0.0),
            ..px(10.0)
        }))]
    );
    assert_eq!(
        declarations("width: calc(var(--space-2) + 2px - var(--space-2));"),
        [Declaration::Width(sized(px(2.0)))]
    );
}

#[test]
fn sizes_take_lengths_percentages_tokens_and_keywords() {
    assert_eq!(
        declarations(
            "width: auto; height: 100%; min-width: 0; min-height: var(--space-8); max-width: none; max-height: 240px;"
        ),
        [
            Declaration::Width(LengthPercentageAuto::Auto),
            Declaration::Height(sized(fraction(1.0))),
            Declaration::MinWidth(sized(px(0.0))),
            Declaration::MinHeight(sized(token("space-8"))),
            Declaration::MaxWidth(LengthPercentageAuto::Auto),
            Declaration::MaxHeight(sized(px(240.0))),
        ]
    );
    assert_eq!(
        declarations("position: absolute; top: -4px; left: auto; overflow-x: hidden; overflow-y: visible;"),
        [
            Declaration::Position(Position::Absolute),
            Declaration::Inset(Side::Top, sized(px(-4.0))),
            Declaration::Inset(Side::Left, LengthPercentageAuto::Auto),
            Declaration::OverflowX(Overflow::Hidden),
            Declaration::OverflowY(Overflow::Visible),
        ]
    );
}

#[test]
fn flex_and_alignment_properties_compile() {
    assert_eq!(
        declarations(
            "flex-direction: row-reverse; flex-wrap: wrap-reverse; flex-grow: 1.5; flex-shrink: 0; flex-basis: 20%; \
             justify-content: space-evenly; align-content: flex-end; justify-items: baseline; align-items: center; \
             justify-self: auto; align-self: flex-start;"
        ),
        [
            Declaration::FlexDirection(FlexDirection::RowReverse),
            Declaration::FlexWrap(FlexWrap::WrapReverse),
            Declaration::FlexGrow(1.5),
            Declaration::FlexShrink(0.0),
            Declaration::FlexBasis(sized(fraction(0.2))),
            Declaration::JustifyContent(ContentAlignment::SpaceEvenly),
            Declaration::AlignContent(ContentAlignment::FlexEnd),
            Declaration::JustifyItems(ItemAlignment::Baseline),
            Declaration::AlignItems(ItemAlignment::Center),
            Declaration::JustifySelf(SelfAlignment::Auto),
            Declaration::AlignSelf(SelfAlignment::Item(ItemAlignment::FlexStart)),
        ]
    );
}

#[test]
fn grid_tracks_compile() {
    let track = |breadth| TrackListItem::Track(TrackSize::Breadth(breadth));

    assert_eq!(
        declarations(
            "grid-template-columns: 190px 1fr minmax(0, 2fr) fit-content(50%) repeat(2, auto min-content) repeat(auto-fit, max-content);"
        ),
        [Declaration::GridTemplateColumns(Cow::Owned(vec![
            track(TrackBreadth::LengthPercentage(px(190.0))),
            track(TrackBreadth::Flex(1.0)),
            TrackListItem::Track(TrackSize::MinMax {
                min: TrackBreadth::LengthPercentage(px(0.0)),
                max: TrackBreadth::Flex(2.0),
            }),
            TrackListItem::Track(TrackSize::FitContent(fraction(0.5))),
            TrackListItem::Repeat {
                count: RepeatCount::Times(2),
                tracks: Cow::Owned(vec![
                    TrackSize::Breadth(TrackBreadth::Auto),
                    TrackSize::Breadth(TrackBreadth::MinContent),
                ]),
            },
            TrackListItem::Repeat {
                count: RepeatCount::AutoFit,
                tracks: Cow::Owned(vec![TrackSize::Breadth(TrackBreadth::MaxContent)]),
            },
        ]))]
    );
    assert_eq!(
        declarations("grid-template-rows: none; grid-auto-rows: var(--space-6) 1fr; grid-auto-columns: auto;"),
        [
            Declaration::GridTemplateRows(Cow::Owned(Vec::new())),
            Declaration::GridAutoRows(Cow::Owned(vec![
                TrackSize::Breadth(TrackBreadth::LengthPercentage(token("space-6"))),
                TrackSize::Breadth(TrackBreadth::Flex(1.0)),
            ])),
            Declaration::GridAutoColumns(Cow::Owned(vec![TrackSize::Breadth(TrackBreadth::Auto)])),
        ]
    );
}

#[test]
fn grid_placement_compiles() {
    assert_eq!(
        declarations(
            "grid-auto-flow: dense; grid-column-start: auto; grid-column-end: -2; grid-row-start: span 2; grid-row-end: 4;"
        ),
        [
            Declaration::GridAutoFlow(GridAutoFlow::RowDense),
            Declaration::GridColumnStart(GridLine::Auto),
            Declaration::GridColumnEnd(GridLine::Line(-2)),
            Declaration::GridRowStart(GridLine::Span(2)),
            Declaration::GridRowEnd(GridLine::Line(4)),
        ]
    );
    assert_eq!(
        declarations("grid-auto-flow: column;"),
        [Declaration::GridAutoFlow(GridAutoFlow::Column)]
    );
    assert_eq!(
        declarations("grid-auto-flow: dense column;"),
        [Declaration::GridAutoFlow(GridAutoFlow::ColumnDense)]
    );
}

#[test]
fn colors_shadows_and_radii_compile() {
    assert_eq!(
        declarations(
            "color: currentColor; background-color: transparent; box-shadow: var(--shadow-float); \
             border-top-left-radius: calc(var(--radius-md) - 2px); outline-offset: -2px;"
        ),
        [
            Declaration::Color(ColorValue::CurrentColor),
            Declaration::BackgroundColor(ColorValue::Transparent),
            Declaration::BoxShadow(BoxShadow::Token(shadow_token("shadow-float"))),
            Declaration::BorderRadius(
                Corner::TopLeft,
                morok_style::Length {
                    pixels: Pixels(-2.0),
                    ..token_length("radius-md")
                }
            ),
            Declaration::OutlineOffset(px_length(-2.0)),
        ]
    );
    assert_eq!(
        declarations("box-shadow: none;"),
        [Declaration::BoxShadow(BoxShadow::None)]
    );
}

#[test]
fn linear_gradients_compile() {
    let stop = |color, position| ColorStop { color, position };
    let gradient = |direction, stops: Vec<ColorStop>| {
        [Declaration::BackgroundImage(BackgroundImage::Gradient(
            Gradient::Linear(LinearGradient {
                direction,
                stops: Cow::Owned(stops),
            }),
        ))]
    };

    assert_eq!(
        declarations("background-image: linear-gradient(var(--color-bg), transparent);"),
        gradient(
            GradientDirection::Angle(180.0),
            vec![stop(color("color-bg"), None), stop(ColorValue::Transparent, None)]
        )
    );
    assert_eq!(
        declarations("background-image: linear-gradient(45deg, var(--color-bg) 10%, currentColor 24px);"),
        gradient(
            GradientDirection::Angle(45.0),
            vec![
                stop(color("color-bg"), Some(fraction(0.1))),
                stop(ColorValue::CurrentColor, Some(px(24.0))),
            ]
        )
    );
    assert_eq!(
        declarations("background-image: linear-gradient(to left, var(--color-bg), var(--color-surface));"),
        gradient(
            GradientDirection::Angle(270.0),
            vec![stop(color("color-bg"), None), stop(color("color-surface"), None)]
        )
    );
    assert_eq!(
        declarations("background-image: linear-gradient(to left top, var(--color-bg), var(--color-surface));"),
        gradient(
            GradientDirection::ToCorner(Corner::TopLeft),
            vec![stop(color("color-bg"), None), stop(color("color-surface"), None)]
        )
    );
}

#[test]
fn radial_gradients_compile() {
    let stops = || {
        Cow::Owned(vec![
            ColorStop {
                color: color("color-bg"),
                position: None,
            },
            ColorStop {
                color: ColorValue::Transparent,
                position: None,
            },
        ])
    };
    let radial = |gradient| {
        [Declaration::BackgroundImage(BackgroundImage::Gradient(
            Gradient::Radial(gradient),
        ))]
    };

    assert_eq!(
        declarations("background-image: radial-gradient(var(--color-bg), transparent);"),
        radial(RadialGradient {
            shape: RadialShape::Ellipse,
            extent: RadialExtent::FarthestCorner,
            center_x: fraction(0.5),
            center_y: fraction(0.5),
            stops: stops(),
        })
    );
    assert_eq!(
        declarations(
            "background-image: radial-gradient(circle closest-side at 12px 25%, var(--color-bg), transparent);"
        ),
        radial(RadialGradient {
            shape: RadialShape::Circle,
            extent: RadialExtent::ClosestSide,
            center_x: px(12.0),
            center_y: fraction(0.25),
            stops: stops(),
        })
    );
    assert_eq!(
        declarations("background-image: radial-gradient(at top right, var(--color-bg), transparent);"),
        radial(RadialGradient {
            shape: RadialShape::Ellipse,
            extent: RadialExtent::FarthestCorner,
            center_x: fraction(1.0),
            center_y: fraction(0.0),
            stops: stops(),
        })
    );
    assert_eq!(
        declarations("background-image: radial-gradient(farthest-side at bottom, var(--color-bg), transparent);"),
        radial(RadialGradient {
            shape: RadialShape::Ellipse,
            extent: RadialExtent::FarthestSide,
            center_x: fraction(0.5),
            center_y: fraction(1.0),
            stops: stops(),
        })
    );
}

#[test]
fn a_background_takes_a_gradient_and_a_color_in_either_order() {
    let both = declarations("background: var(--color-bg) linear-gradient(var(--color-bg), transparent);");

    assert_eq!(
        both,
        declarations("background: linear-gradient(var(--color-bg), transparent) var(--color-bg);")
    );
    assert_eq!(both[0], Declaration::BackgroundColor(color("color-bg")));
    assert!(matches!(
        both[1],
        Declaration::BackgroundImage(BackgroundImage::Gradient(Gradient::Linear(_)))
    ));
    assert_eq!(
        declarations("background: none;"),
        [
            Declaration::BackgroundColor(ColorValue::Transparent),
            Declaration::BackgroundImage(BackgroundImage::None),
        ]
    );
}

#[test]
fn a_later_declaration_of_a_property_stays_after_the_earlier_one() {
    assert_eq!(
        declarations("display: flex; display: grid;"),
        [Declaration::Display(Display::Flex), Declaration::Display(Display::Grid)]
    );
}
