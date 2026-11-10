use cssparser::{Parser, Token};
use morok_style::{
    AnimatableProperties, AnimatableProperty, AnimationDirection, AnimationName, Corner, Declaration, Easing,
    EasingValue, FillMode, IterationCount, Milliseconds, Side, TimeValue,
};

use crate::problem::Problem;
use crate::property::{self, Property};
use crate::property_syntax::{comma_separated, keyword, peek};
use crate::span::Span;
use crate::token_id::TokenId;
use crate::token_reference;
use crate::value_context::ValueContext;
use crate::value_error::{ValueError, ValueResult};
use crate::value_syntax::motion::easing;

const MILLISECONDS_PER_SECOND: f32 = 1000.0;
const NONE: &str = "none";
const ALL: &str = "all";
const INFINITE: &str = "infinite";
/// The CSS initial easing, `ease`.
const EASE: Easing = Easing::CubicBezier {
    x1: 0.25,
    y1: 0.1,
    x2: 0.25,
    y2: 1.0,
};
const NO_TIME: TimeValue = TimeValue::Milliseconds(Milliseconds(0.0));
const PLAY_STATES: [&str; 2] = ["running", "paused"];
const DIRECTIONS: [(&str, AnimationDirection); 4] = [
    ("normal", AnimationDirection::Normal),
    ("reverse", AnimationDirection::Reverse),
    ("alternate", AnimationDirection::Alternate),
    ("alternate-reverse", AnimationDirection::AlternateReverse),
];
const FILL_MODES: [(&str, FillMode); 4] = [
    ("none", FillMode::None),
    ("forwards", FillMode::Forwards),
    ("backwards", FillMode::Backwards),
    ("both", FillMode::Both),
];

pub(crate) const PROPERTIES: &[Property] = &[
    Property {
        name: "transition",
        expectation: "`none` or a list of transitions, each a property, a duration, an easing and a delay",
        parse: |context, input| transition(context, input),
    },
    Property {
        name: "animation",
        expectation: "`none` or a list of animations, each the name of keyframes, a duration, an easing, a delay, \
                      an iteration count, a direction and a fill mode",
        parse: |context, input| animation(context, input),
    },
];

/// A time or an easing of a list item, written out or as a `motion-` token.
enum MotionTerm {
    Time(TimeValue),
    Easing(EasingValue),
}

/// Reads a time or an easing when one is ahead; a time may be negative. A token of another
/// family is reported.
fn motion_term(context: &ValueContext<'_>, input: &mut Parser<'_>) -> ValueResult<Option<MotionTerm>> {
    if token_reference::is_ahead(input) {
        return Ok(Some(match token_reference::parse(context.tokens, input)?.motion()? {
            TokenId::Duration(token) => MotionTerm::Time(TimeValue::Token(token)),
            TokenId::Easing(token) => MotionTerm::Easing(EasingValue::Token(token)),
            _ => unreachable!("a motion token is a duration or an easing"),
        }));
    }
    if let Ok(milliseconds) = input.try_parse(time) {
        return Ok(Some(MotionTerm::Time(TimeValue::Milliseconds(milliseconds))));
    }
    if let Ok(curve) = input.try_parse(easing) {
        return Ok(Some(MotionTerm::Easing(EasingValue::Curve(curve))));
    }
    Ok(None)
}

fn time(input: &mut Parser<'_>) -> Result<Milliseconds, ()> {
    let milliseconds = match input.next().map_err(|_| ())?.clone() {
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("ms") => value,
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("s") => value * MILLISECONDS_PER_SECOND,
        _ => return Err(()),
    };
    if milliseconds.is_finite() {
        Ok(Milliseconds(milliseconds))
    } else {
        Err(())
    }
}

/// The two times of a list item: the first is the duration, the second the delay.
#[derive(Default)]
struct Times {
    duration: Option<TimeValue>,
    delay: Option<TimeValue>,
}

impl Times {
    fn add(&mut self, time: TimeValue, span: Span) -> ValueResult<()> {
        if self.duration.is_none() {
            if matches!(time, TimeValue::Milliseconds(Milliseconds(milliseconds)) if milliseconds < 0.0) {
                return Err(Problem {
                    span,
                    message: "a duration cannot be negative".to_owned(),
                    hint: "the first time is the duration and the second the delay, which may be negative".to_owned(),
                }
                .into());
            }
            self.duration = Some(time);
        } else if self.delay.is_none() {
            self.delay = Some(time);
        } else {
            return Err(ValueError::Mismatch);
        }
        Ok(())
    }
}

/// Stores the value of a part of a list item that may be written once.
fn once<T>(slot: &mut Option<T>, value: T) -> ValueResult<()> {
    if slot.is_some() {
        return Err(ValueError::Mismatch);
    }
    *slot = Some(value);
    Ok(())
}

struct SingleTransition {
    properties: AnimatableProperties,
    duration: TimeValue,
    easing: EasingValue,
    delay: TimeValue,
}

fn transition(context: &ValueContext<'_>, input: &mut Parser<'_>) -> ValueResult<Vec<Declaration>> {
    let transitions = comma_separated(input, |item| single_transition(context, item))?;
    if transitions.len() > 1 && transitions.iter().any(|transition| transition.properties.is_empty()) {
        return Err(ValueError::Mismatch);
    }
    Ok(vec![
        Declaration::TransitionProperty(transitions.iter().map(|transition| transition.properties).collect()),
        Declaration::TransitionDuration(transitions.iter().map(|transition| transition.duration).collect()),
        Declaration::TransitionTimingFunction(transitions.iter().map(|transition| transition.easing).collect()),
        Declaration::TransitionDelay(transitions.iter().map(|transition| transition.delay).collect()),
    ])
}

fn single_transition(context: &ValueContext<'_>, input: &mut Parser<'_>) -> ValueResult<SingleTransition> {
    let mut properties = None;
    let mut times = Times::default();
    let mut curve = None;
    while !input.is_exhausted() {
        input.skip_whitespace();
        let start = input.position();
        match motion_term(context, input)? {
            Some(MotionTerm::Time(time)) => times.add(time, Span::between(start, input.position()))?,
            Some(MotionTerm::Easing(easing)) => once(&mut curve, easing)?,
            None => once(&mut properties, transitioned_properties(input)?)?,
        }
    }
    Ok(SingleTransition {
        properties: properties.unwrap_or(AnimatableProperties::ALL),
        duration: times.duration.unwrap_or(NO_TIME),
        easing: curve.unwrap_or(EasingValue::Curve(EASE)),
        delay: times.delay.unwrap_or(NO_TIME),
    })
}

/// The properties one transition names: `none`, `all`, a longhand or a shorthand.
fn transitioned_properties(input: &mut Parser<'_>) -> ValueResult<AnimatableProperties> {
    input.skip_whitespace();
    let start = input.position();
    let name = input.expect_ident_cloned().map_err(|_| ValueError::Mismatch)?;
    if let Some(properties) = named_properties(&name) {
        return Ok(properties);
    }
    let span = Span::between(start, input.position());
    let message = if property::named(&name).is_some() {
        format!("`{name}` cannot be transitioned")
    } else {
        format!("`{name}` is not a property")
    };
    Err(Problem {
        span,
        message,
        hint: "a transition runs on lengths, colors, `box-shadow`, `opacity` and `transform`".to_owned(),
    }
    .into())
}

fn sides(property: fn(Side) -> AnimatableProperty) -> AnimatableProperties {
    Side::ALL.into_iter().fold(AnimatableProperties::NONE, |set, side| {
        set.union(AnimatableProperties::of(property(side)))
    })
}

fn side_border(side: Side) -> AnimatableProperties {
    AnimatableProperties::of(AnimatableProperty::BorderWidth(side))
        .union(AnimatableProperties::of(AnimatableProperty::BorderColor(side)))
}

fn one(property: AnimatableProperty) -> AnimatableProperties {
    AnimatableProperties::of(property)
}

/// The animatable properties a name of `transition-property` stands for.
fn named_properties(name: &str) -> Option<AnimatableProperties> {
    let name = name.to_ascii_lowercase();
    Some(match name.as_str() {
        NONE => AnimatableProperties::NONE,
        ALL => AnimatableProperties::ALL,
        "width" => one(AnimatableProperty::Width),
        "height" => one(AnimatableProperty::Height),
        "min-width" => one(AnimatableProperty::MinWidth),
        "min-height" => one(AnimatableProperty::MinHeight),
        "max-width" => one(AnimatableProperty::MaxWidth),
        "max-height" => one(AnimatableProperty::MaxHeight),
        "margin" => sides(AnimatableProperty::Margin),
        "margin-top" => one(AnimatableProperty::Margin(Side::Top)),
        "margin-right" => one(AnimatableProperty::Margin(Side::Right)),
        "margin-bottom" => one(AnimatableProperty::Margin(Side::Bottom)),
        "margin-left" => one(AnimatableProperty::Margin(Side::Left)),
        "padding" => sides(AnimatableProperty::Padding),
        "padding-top" => one(AnimatableProperty::Padding(Side::Top)),
        "padding-right" => one(AnimatableProperty::Padding(Side::Right)),
        "padding-bottom" => one(AnimatableProperty::Padding(Side::Bottom)),
        "padding-left" => one(AnimatableProperty::Padding(Side::Left)),
        "inset" => sides(AnimatableProperty::Inset),
        "top" => one(AnimatableProperty::Inset(Side::Top)),
        "right" => one(AnimatableProperty::Inset(Side::Right)),
        "bottom" => one(AnimatableProperty::Inset(Side::Bottom)),
        "left" => one(AnimatableProperty::Inset(Side::Left)),
        "gap" => one(AnimatableProperty::RowGap).union(one(AnimatableProperty::ColumnGap)),
        "row-gap" => one(AnimatableProperty::RowGap),
        "column-gap" => one(AnimatableProperty::ColumnGap),
        "border" => sides(AnimatableProperty::BorderWidth).union(sides(AnimatableProperty::BorderColor)),
        "border-top" => side_border(Side::Top),
        "border-right" => side_border(Side::Right),
        "border-bottom" => side_border(Side::Bottom),
        "border-left" => side_border(Side::Left),
        "border-width" => sides(AnimatableProperty::BorderWidth),
        "border-top-width" => one(AnimatableProperty::BorderWidth(Side::Top)),
        "border-right-width" => one(AnimatableProperty::BorderWidth(Side::Right)),
        "border-bottom-width" => one(AnimatableProperty::BorderWidth(Side::Bottom)),
        "border-left-width" => one(AnimatableProperty::BorderWidth(Side::Left)),
        "border-color" => sides(AnimatableProperty::BorderColor),
        "border-top-color" => one(AnimatableProperty::BorderColor(Side::Top)),
        "border-right-color" => one(AnimatableProperty::BorderColor(Side::Right)),
        "border-bottom-color" => one(AnimatableProperty::BorderColor(Side::Bottom)),
        "border-left-color" => one(AnimatableProperty::BorderColor(Side::Left)),
        "border-radius" => Corner::ALL.into_iter().fold(AnimatableProperties::NONE, |set, corner| {
            set.union(one(AnimatableProperty::BorderRadius(corner)))
        }),
        "border-top-left-radius" => one(AnimatableProperty::BorderRadius(Corner::TopLeft)),
        "border-top-right-radius" => one(AnimatableProperty::BorderRadius(Corner::TopRight)),
        "border-bottom-right-radius" => one(AnimatableProperty::BorderRadius(Corner::BottomRight)),
        "border-bottom-left-radius" => one(AnimatableProperty::BorderRadius(Corner::BottomLeft)),
        "outline" => one(AnimatableProperty::OutlineWidth).union(one(AnimatableProperty::OutlineColor)),
        "outline-width" => one(AnimatableProperty::OutlineWidth),
        "outline-color" => one(AnimatableProperty::OutlineColor),
        "outline-offset" => one(AnimatableProperty::OutlineOffset),
        "color" => one(AnimatableProperty::Color),
        "background" | "background-color" => one(AnimatableProperty::BackgroundColor),
        "box-shadow" => one(AnimatableProperty::BoxShadow),
        "opacity" => one(AnimatableProperty::Opacity),
        "transform" => one(AnimatableProperty::Transform),
        _ => return None,
    })
}

struct SingleAnimation {
    name: AnimationName,
    duration: TimeValue,
    easing: EasingValue,
    delay: TimeValue,
    iteration_count: IterationCount,
    direction: AnimationDirection,
    fill_mode: FillMode,
}

fn animation(context: &ValueContext<'_>, input: &mut Parser<'_>) -> ValueResult<Vec<Declaration>> {
    let animations = comma_separated(input, |item| single_animation(context, item))?;
    Ok(vec![
        Declaration::AnimationName(animations.iter().map(|animation| animation.name).collect()),
        Declaration::AnimationDuration(animations.iter().map(|animation| animation.duration).collect()),
        Declaration::AnimationTimingFunction(animations.iter().map(|animation| animation.easing).collect()),
        Declaration::AnimationDelay(animations.iter().map(|animation| animation.delay).collect()),
        Declaration::AnimationIterationCount(animations.iter().map(|animation| animation.iteration_count).collect()),
        Declaration::AnimationDirection(animations.iter().map(|animation| animation.direction).collect()),
        Declaration::AnimationFillMode(animations.iter().map(|animation| animation.fill_mode).collect()),
    ])
}

/// Reads the parts of one animation in any order. A keyword goes to the first part that takes
/// it and is not yet written, the name last: `none` is the fill mode first and the name second.
fn single_animation(context: &ValueContext<'_>, input: &mut Parser<'_>) -> ValueResult<SingleAnimation> {
    let mut times = Times::default();
    let mut curve = None;
    let mut iteration_count = None;
    let mut direction = None;
    let mut fill_mode = None;
    let mut name = None;
    while !input.is_exhausted() {
        input.skip_whitespace();
        let start = input.position();
        if let Some(term) = motion_term(context, input)? {
            match term {
                MotionTerm::Time(time) => times.add(time, Span::between(start, input.position()))?,
                MotionTerm::Easing(easing) => once(&mut curve, easing)?,
            }
            continue;
        }
        if iteration_count.is_none()
            && let Ok(count) = input.try_parse(iterations)
        {
            iteration_count = Some(count);
            continue;
        }
        if direction.is_none()
            && let Ok(value) = input.try_parse(|input| keyword(input, &DIRECTIONS))
        {
            direction = Some(value);
            continue;
        }
        if fill_mode.is_none()
            && let Ok(value) = input.try_parse(|input| keyword(input, &FILL_MODES))
        {
            fill_mode = Some(value);
            continue;
        }
        once(&mut name, animation_name(context, input)?)?;
    }
    Ok(SingleAnimation {
        name: name.unwrap_or(AnimationName::None),
        duration: times.duration.unwrap_or(NO_TIME),
        easing: curve.unwrap_or(EasingValue::Curve(EASE)),
        delay: times.delay.unwrap_or(NO_TIME),
        iteration_count: iteration_count.unwrap_or(IterationCount::Count(1.0)),
        direction: direction.unwrap_or(AnimationDirection::Normal),
        fill_mode: fill_mode.unwrap_or(FillMode::None),
    })
}

fn iterations(input: &mut Parser<'_>) -> ValueResult<IterationCount> {
    match input.next().map_err(|_| ValueError::Mismatch)?.clone() {
        Token::Ident(ident) if ident.eq_ignore_ascii_case(INFINITE) => Ok(IterationCount::Infinite),
        Token::Number { value, .. } if value >= 0.0 && value.is_finite() => Ok(IterationCount::Count(value)),
        _ => Err(ValueError::Mismatch),
    }
}

/// `none` or the name of a `@keyframes` rule of the stylesheets.
fn animation_name(context: &ValueContext<'_>, input: &mut Parser<'_>) -> ValueResult<AnimationName> {
    let Some(Token::Ident(_)) = peek(input) else {
        return Err(ValueError::Mismatch);
    };
    input.skip_whitespace();
    let start = input.position();
    let name = input.expect_ident_cloned().map_err(|_| ValueError::Mismatch)?;
    let span = Span::between(start, input.position());
    if name.eq_ignore_ascii_case(NONE) {
        return Ok(AnimationName::None);
    }
    if PLAY_STATES.iter().any(|state| name.eq_ignore_ascii_case(state)) {
        return Err(Problem {
            span,
            message: format!("`{name}` is not supported"),
            hint: "an animation runs from the moment its element is styled; it cannot be paused".to_owned(),
        }
        .into());
    }
    match context.keyframes.id(&name) {
        Some(keyframes) => Ok(AnimationName::Keyframes(keyframes)),
        None => Err(Problem {
            span,
            message: format!("no `@keyframes` rule is named `{name}`"),
            hint: "declare it in one of the stylesheets compiled together".to_owned(),
        }
        .into()),
    }
}
