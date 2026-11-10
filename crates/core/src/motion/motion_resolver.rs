use std::mem;

use morok_style::{
    AnimatableProperties, AnimatableProperty, AnimationName, Color, Declaration, Easing, KeyframesRule, StyleSheet,
    Theme,
};

use crate::element::Element;
use crate::motion::animation::Animation;
use crate::motion::declared_value::declared_value;
use crate::motion::easing_curve::eased;
use crate::motion::element_motion::ElementMotion;
use crate::motion::held_value::HeldValue;
use crate::motion::interpolation::{interpolate, is_interpolable};
use crate::motion::iteration::Iteration;
use crate::motion::motion_declarations::MotionDeclarations;
use crate::motion::motion_scratch::MotionScratch;
use crate::motion::property_value::PropertyValue;
use crate::motion::timeline::Timeline;
use crate::motion::transition::Transition;
use crate::motion::transition_timing::TransitionTiming;
use crate::style::computed_style::ComputedStyle;
use crate::style::style_lists::StyleLists;

/// Works out the motion of elements at the time of the frame: which transitions and keyframe
/// animations run, and what an element shows while they do.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MotionResolver<'a> {
    pub(crate) style_sheet: &'a StyleSheet,
    pub(crate) theme: &'a Theme,
    pub(crate) timeline: Timeline,
}

/// What an element showed before it was styled again.
#[derive(Clone, Copy, Debug)]
struct ShownBefore<'a> {
    style: &'a ComputedStyle,
    lists: &'a StyleLists,
    /// Whether the element was rendered, and so has a style a transition can start from.
    was_rendered: bool,
}

impl MotionResolver<'_> {
    /// The element was styled again into `base`: its animations follow `animation-name`, a
    /// transition starts for every transitioned property whose value changed and stops for one
    /// no longer transitioned. An element that is not rendered runs no motion at all.
    pub(crate) fn restart(
        &self,
        element: &mut Element,
        base: &ComputedStyle,
        base_lists: &StyleLists,
        is_rendered: bool,
        scratch: &mut MotionScratch,
    ) {
        let Element {
            motion,
            style,
            lists,
            inline_style,
            is_rendered: was_rendered,
            ..
        } = element;
        if !is_rendered {
            if let Some(motion) = motion {
                motion.clear();
            }
            return;
        }
        let declarations = MotionDeclarations {
            motion: base.motion,
            style_sheet: self.style_sheet,
            inline_style,
            theme: self.theme,
        };
        self.follow_animation_names(motion, base, &declarations, scratch);
        let transitions_run = motion.as_ref().is_some_and(|motion| !motion.transitions.is_empty());
        if base.motion.transition_property.is_some() || transitions_run {
            let before = ShownBefore {
                style,
                lists,
                was_rendered: *was_rendered,
            };
            let animated = motion
                .as_deref()
                .map_or(AnimatableProperties::NONE, |motion| self.animated_properties(motion));
            for property in AnimatableProperty::ALL {
                if !animated.contains(property) {
                    self.retransition(property, motion, &declarations, before, base, base_lists, scratch);
                }
            }
        }
        if let Some(motion) = motion {
            motion.base = *base;
            motion.base_lists.copy_from(base_lists);
        }
    }

    /// Writes into `shown` and `shown_lists` what the element shows now: `base` with the values
    /// of its animations and, over them, of its transitions; a color that is `currentColor`
    /// follows the color shown. Transitions that ended are dropped. Returns whether a later
    /// frame shows something else.
    pub(crate) fn show(
        &self,
        motion: Option<&mut ElementMotion>,
        base: &ComputedStyle,
        base_lists: &StyleLists,
        shown: &mut ComputedStyle,
        shown_lists: &mut StyleLists,
        scratch: &mut MotionScratch,
    ) -> bool {
        *shown = *base;
        shown_lists.copy_from(base_lists);
        let Some(motion) = motion else {
            return false;
        };
        motion
            .transitions
            .retain(|transition| !transition.is_finished(&self.timeline));
        let mut written = AnimatableProperties::NONE;
        for animation in &motion.animations {
            let Some(iteration) = animation.iteration_now(&self.timeline) else {
                continue;
            };
            let rule = self.style_sheet.keyframes_rule(animation.keyframes);
            for property in keyframes_properties(rule).iter() {
                let current_color = shown.paint.color;
                self.keyframes_value(
                    rule,
                    property,
                    iteration,
                    &animation.timing.easing,
                    (base, base_lists),
                    current_color,
                    scratch,
                );
                scratch.value.value().write(property, shown, shown_lists);
                written = written.union(AnimatableProperties::of(property));
            }
        }
        for transition in &motion.transitions {
            transition.value_now(&self.timeline, &mut scratch.value);
            scratch.value.value().write(transition.property, shown, shown_lists);
            written = written.union(AnimatableProperties::of(transition.property));
        }
        if shown.paint.color != base.paint.color {
            for property in base.paint.follows_color.iter() {
                if !written.contains(property) {
                    PropertyValue::Color(shown.paint.color).write(property, shown, shown_lists);
                }
            }
        }
        motion.is_running(&self.timeline)
    }

    /// Lines the animations up with `animation-name`, matching names from the last to the first:
    /// a name that runs already keeps its start and takes the new timing, a new name starts now,
    /// and a name dropped stops. Under reduced motion an animation that never ends does not run.
    fn follow_animation_names(
        &self,
        motion: &mut Option<Box<ElementMotion>>,
        base: &ComputedStyle,
        declarations: &MotionDeclarations<'_>,
        scratch: &mut MotionScratch,
    ) {
        let names = declarations.animation_names();
        let names_keyframes = names.iter().any(|name| matches!(name, AnimationName::Keyframes(_)));
        let motion = if names_keyframes {
            motion.get_or_insert_with(|| Box::new(ElementMotion::new(base)))
        } else if let Some(motion) = motion {
            motion
        } else {
            return;
        };
        scratch.animations.clear();
        for (position, name) in names.iter().enumerate().rev() {
            let AnimationName::Keyframes(keyframes) = *name else {
                continue;
            };
            let timing = declarations.animation_timing(position);
            if self.timeline.reduces_motion && timing.iterations.is_infinite() {
                continue;
            }
            let start_time = match motion
                .animations
                .iter()
                .rposition(|animation| animation.keyframes == keyframes)
            {
                Some(running) => motion.animations.remove(running).start_time,
                None => self.timeline.now,
            };
            scratch.animations.push(Animation {
                keyframes,
                start_time,
                timing,
            });
        }
        scratch.animations.reverse();
        mem::swap(&mut motion.animations, &mut scratch.animations);
    }

    /// The properties the animations of the element set now.
    fn animated_properties(&self, motion: &ElementMotion) -> AnimatableProperties {
        motion
            .animations
            .iter()
            .filter(|animation| animation.iteration_now(&self.timeline).is_some())
            .fold(AnimatableProperties::NONE, |properties, animation| {
                properties.union(keyframes_properties(
                    self.style_sheet.keyframes_rule(animation.keyframes),
                ))
            })
    }

    /// Starts, reverses, replaces or stops the transition of one property as CSS Transitions
    /// does when the style of an element changes from `before` to `base`.
    #[allow(clippy::too_many_arguments)]
    fn retransition(
        &self,
        property: AnimatableProperty,
        motion: &mut Option<Box<ElementMotion>>,
        declarations: &MotionDeclarations<'_>,
        before: ShownBefore<'_>,
        base: &ComputedStyle,
        base_lists: &StyleLists,
        scratch: &mut MotionScratch,
    ) {
        let follows_color =
            before.style.paint.follows_color.contains(property) && base.paint.follows_color.contains(property);
        let timing = declarations.transition_timing(property).filter(|timing| {
            timing.takes_time() && before.was_rendered && !follows_color && !self.timeline.reduces_motion
        });
        let shown_before = PropertyValue::read(property, before.style, before.lists);
        let after = PropertyValue::read(property, base, base_lists);
        let running = motion.as_ref().and_then(|motion| {
            motion
                .transitions
                .iter()
                .position(|transition| transition.property == property)
        });
        let Some(running) = running else {
            let Some(timing) = timing else {
                return;
            };
            if shown_before == after || !is_interpolable(shown_before, after) {
                return;
            }
            let motion = motion.get_or_insert_with(|| Box::new(ElementMotion::new(base)));
            motion.transitions.push(self.transition(
                property,
                shown_before,
                after,
                HeldValue::of(shown_before),
                1.0,
                timing,
            ));
            return;
        };
        let motion = motion
            .as_mut()
            .expect("a running transition belongs to the motion of its element");
        let Some(timing) = timing else {
            motion.transitions.remove(running);
            return;
        };
        let transition = &motion.transitions[running];
        if transition.end_value.value() == after {
            return;
        }
        transition.value_now(&self.timeline, &mut scratch.current);
        let current = scratch.current.value();
        if current == after || !is_interpolable(current, after) {
            motion.transitions.remove(running);
            return;
        }
        let replacement = if transition.reversing_adjusted_start_value.value() == after {
            let previous_factor = transition.reversing_shortening_factor;
            let factor = (transition.output_progress(&self.timeline) * previous_factor + 1.0 - previous_factor)
                .abs()
                .clamp(0.0, 1.0);
            let shortened = TransitionTiming {
                duration: timing.duration * factor,
                delay: if timing.delay < 0.0 {
                    timing.delay * factor
                } else {
                    timing.delay
                },
                easing: timing.easing,
            };
            self.transition(
                property,
                current,
                after,
                transition.end_value.clone(),
                factor,
                shortened,
            )
        } else {
            self.transition(property, current, after, HeldValue::of(current), 1.0, timing)
        };
        motion.transitions[running] = replacement;
    }

    fn transition(
        &self,
        property: AnimatableProperty,
        start: PropertyValue<'_>,
        end: PropertyValue<'_>,
        reversing_adjusted_start_value: HeldValue,
        reversing_shortening_factor: f64,
        timing: TransitionTiming,
    ) -> Transition {
        Transition {
            property,
            start_value: HeldValue::of(start),
            end_value: HeldValue::of(end),
            reversing_adjusted_start_value,
            reversing_shortening_factor,
            start_time: self.timeline.now,
            timing,
        }
    }

    /// Writes into `scratch.value` the value the keyframes give the property at the iteration:
    /// interpolated between the keyframes around its progress, with the interval eased. Where no
    /// keyframe at the start or the end sets the property, the base value stands in.
    #[allow(clippy::too_many_arguments)]
    fn keyframes_value(
        &self,
        rule: &KeyframesRule,
        property: AnimatableProperty,
        iteration: Iteration,
        easing: &Easing,
        (base, base_lists): (&ComputedStyle, &StyleLists),
        current_color: Color,
        scratch: &mut MotionScratch,
    ) {
        let progress = iteration.progress;
        let is_reached = |offset: f32| {
            if progress >= 1.0 {
                offset < 1.0
            } else {
                f64::from(offset) <= progress
            }
        };
        let mut from = (0.0, None);
        let mut to = (1.0, None);
        for keyframe in rule.keyframes.iter() {
            let Some(declaration) = declaration_of(&keyframe.declarations, property) else {
                continue;
            };
            if is_reached(keyframe.offset) {
                from = (keyframe.offset, Some(declaration));
            } else {
                to = (keyframe.offset, Some(declaration));
                break;
            }
        }
        let MotionScratch {
            value,
            from_steps,
            to_steps,
            ..
        } = scratch;
        let base_value = PropertyValue::read(property, base, base_lists);
        let from_value = from.1.map_or(base_value, |declaration| {
            declared_value(declaration, self.theme, current_color, from_steps)
        });
        let to_value = to.1.map_or(base_value, |declaration| {
            declared_value(declaration, self.theme, current_color, to_steps)
        });
        let local = (progress - f64::from(from.0)) / (f64::from(to.0) - f64::from(from.0));
        interpolate(
            property,
            from_value,
            to_value,
            eased(easing, local, iteration.is_before),
            value,
        );
    }
}

/// The last declaration of the property among `declarations`.
fn declaration_of(declarations: &[Declaration], property: AnimatableProperty) -> Option<&Declaration> {
    declarations
        .iter()
        .rev()
        .find(|declaration| declaration.animatable_property() == Some(property))
}

/// The properties some keyframe of the rule sets.
fn keyframes_properties(rule: &KeyframesRule) -> AnimatableProperties {
    rule.keyframes
        .iter()
        .flat_map(|keyframe| keyframe.declarations.iter())
        .filter_map(Declaration::animatable_property)
        .fold(AnimatableProperties::NONE, |properties, property| {
            properties.union(AnimatableProperties::of(property))
        })
}
