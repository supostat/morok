use morok_style::{
    AnimatableProperties, AnimatableProperty, AnimationDirection, AnimationName, Declaration, Easing, EasingValue,
    FillMode, IterationCount, Milliseconds, StyleSheet, Theme, TimeValue,
};

use crate::motion::animation_timing::AnimationTiming;
use crate::motion::transition_timing::TransitionTiming;
use crate::style::declaration_source::DeclarationSource;
use crate::style::inline_style::InlineStyle;
use crate::style::motion_style::MotionStyle;

/// The CSS initial easing, `ease`.
const EASE: Easing = Easing::CubicBezier {
    x1: 0.25,
    y1: 0.1,
    x2: 0.25,
    y2: 1.0,
};
const INITIAL_TRANSITION_PROPERTY: [AnimatableProperties; 1] = [AnimatableProperties::ALL];
const INITIAL_TIME: [TimeValue; 1] = [TimeValue::Milliseconds(Milliseconds(0.0))];
const INITIAL_EASING: [EasingValue; 1] = [EasingValue::Curve(EASE)];
const INITIAL_ITERATION_COUNT: [IterationCount; 1] = [IterationCount::Count(1.0)];
const INITIAL_DIRECTION: [AnimationDirection; 1] = [AnimationDirection::Normal];
const INITIAL_FILL_MODE: [FillMode; 1] = [FillMode::None];

/// The lists of the motion longhands of an element read where they are declared, with the
/// timings they make for one transition or one animation.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MotionDeclarations<'a> {
    pub(crate) motion: MotionStyle,
    pub(crate) style_sheet: &'a StyleSheet,
    pub(crate) inline_style: &'a InlineStyle,
    pub(crate) theme: &'a Theme,
}

impl<'a> MotionDeclarations<'a> {
    /// The timing of the transitions of the property; none when no transition names it. The
    /// last entry of `transition-property` that names it wins, and every other list repeats
    /// to the length of that one.
    pub(crate) fn transition_timing(&self, property: AnimatableProperty) -> Option<TransitionTiming> {
        let properties = self.list(
            self.motion.transition_property,
            &INITIAL_TRANSITION_PROPERTY,
            |declaration| match declaration {
                Declaration::TransitionProperty(properties) => properties,
                other => unreachable!("transition-property is read from {other:?}"),
            },
        );
        let position = properties.iter().rposition(|entry| entry.contains(property))?;
        let durations = self.times(self.motion.transition_duration, |declaration| match declaration {
            Declaration::TransitionDuration(times) => times,
            other => unreachable!("transition-duration is read from {other:?}"),
        });
        let easings = self.easings(
            self.motion.transition_timing_function,
            |declaration| match declaration {
                Declaration::TransitionTimingFunction(easings) => easings,
                other => unreachable!("transition-timing-function is read from {other:?}"),
            },
        );
        let delays = self.times(self.motion.transition_delay, |declaration| match declaration {
            Declaration::TransitionDelay(times) => times,
            other => unreachable!("transition-delay is read from {other:?}"),
        });
        Some(TransitionTiming {
            duration: self.milliseconds(durations[position % durations.len()]),
            delay: self.milliseconds(delays[position % delays.len()]),
            easing: self.easing(easings[position % easings.len()]),
        })
    }

    pub(crate) fn animation_names(&self) -> &'a [AnimationName] {
        let Some(source) = self.motion.animation_name else {
            return &[];
        };
        match source.declaration(self.style_sheet, self.inline_style) {
            Declaration::AnimationName(names) => names,
            other => unreachable!("animation-name is read from {other:?}"),
        }
    }

    /// The timing of the animation at `position` in `animation-name`; every other list repeats
    /// to the length of that one.
    pub(crate) fn animation_timing(&self, position: usize) -> AnimationTiming {
        let durations = self.times(self.motion.animation_duration, |declaration| match declaration {
            Declaration::AnimationDuration(times) => times,
            other => unreachable!("animation-duration is read from {other:?}"),
        });
        let easings = self.easings(self.motion.animation_timing_function, |declaration| match declaration {
            Declaration::AnimationTimingFunction(easings) => easings,
            other => unreachable!("animation-timing-function is read from {other:?}"),
        });
        let delays = self.times(self.motion.animation_delay, |declaration| match declaration {
            Declaration::AnimationDelay(times) => times,
            other => unreachable!("animation-delay is read from {other:?}"),
        });
        let iteration_counts = self.list(
            self.motion.animation_iteration_count,
            &INITIAL_ITERATION_COUNT,
            |declaration| match declaration {
                Declaration::AnimationIterationCount(counts) => counts,
                other => unreachable!("animation-iteration-count is read from {other:?}"),
            },
        );
        let directions = self.list(
            self.motion.animation_direction,
            &INITIAL_DIRECTION,
            |declaration| match declaration {
                Declaration::AnimationDirection(directions) => directions,
                other => unreachable!("animation-direction is read from {other:?}"),
            },
        );
        let fill_modes = self.list(
            self.motion.animation_fill_mode,
            &INITIAL_FILL_MODE,
            |declaration| match declaration {
                Declaration::AnimationFillMode(fill_modes) => fill_modes,
                other => unreachable!("animation-fill-mode is read from {other:?}"),
            },
        );
        AnimationTiming {
            duration: self.milliseconds(durations[position % durations.len()]),
            delay: self.milliseconds(delays[position % delays.len()]),
            iterations: match iteration_counts[position % iteration_counts.len()] {
                IterationCount::Count(count) => f64::from(count),
                IterationCount::Infinite => f64::INFINITY,
            },
            direction: directions[position % directions.len()],
            fill_mode: fill_modes[position % fill_modes.len()],
            easing: self.easing(easings[position % easings.len()]),
        }
    }

    fn list<T>(
        &self,
        source: Option<DeclarationSource>,
        initial: &'static [T],
        read: impl FnOnce(&'a Declaration) -> &'a [T],
    ) -> &'a [T] {
        source.map_or(initial, |source| {
            read(source.declaration(self.style_sheet, self.inline_style))
        })
    }

    fn times(
        &self,
        source: Option<DeclarationSource>,
        read: impl FnOnce(&'a Declaration) -> &'a [TimeValue],
    ) -> &'a [TimeValue] {
        self.list(source, &INITIAL_TIME, read)
    }

    fn easings(
        &self,
        source: Option<DeclarationSource>,
        read: impl FnOnce(&'a Declaration) -> &'a [EasingValue],
    ) -> &'a [EasingValue] {
        self.list(source, &INITIAL_EASING, read)
    }

    fn milliseconds(&self, time: TimeValue) -> f64 {
        match time {
            TimeValue::Milliseconds(milliseconds) => f64::from(milliseconds.0),
            TimeValue::Token(token) => f64::from(self.theme.resolve(token).0),
        }
    }

    fn easing(&self, easing: EasingValue) -> Easing {
        match easing {
            EasingValue::Curve(curve) => curve,
            EasingValue::Token(token) => *self.theme.resolve(token),
        }
    }
}
