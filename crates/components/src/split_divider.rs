use std::ops::RangeInclusive;

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{
    AccessibilityAction, AccessibleActions, ElementId, Focusability, Key, KeyEvent, KeyPhase, NumericValue,
    PointerButton, PointerEvent, PointerEventKind, Propagation,
};
use morok_reactive::{Runtime, Signal};

use crate::split_axis::SplitAxis;
use crate::styles::CLASSES;
use crate::view::{ElementEvent, Mount, Property, Surfaces};

/// How far an arrow key or an assistive technology moves a divider at once.
const STEP: f32 = 16.0;

/// The divider between two panes of a split view: a line across the axis of the view that the
/// pointer drags and the arrow keys of the axis move. It sets the length of the pane it belongs
/// to, within the limits of that pane, and folds a pane that can fold.
#[derive(Clone, Debug)]
pub(crate) struct SplitDivider {
    /// What an assistive technology calls the divider.
    pub(crate) label: Property<String>,
    pub(crate) axis: SplitAxis,
    pub(crate) length: Signal<f32>,
    pub(crate) limits: RangeInclusive<f32>,
    pub(crate) is_collapsed: Option<Signal<bool>>,
    /// Whether the pane lies before the divider, so that moving the divider along the axis
    /// lengthens it.
    pub(crate) pane_lies_before: bool,
}

/// A drag of a divider: where along the axis the pointer went down and how long the pane was
/// then.
#[derive(Clone, Copy, Debug)]
struct Drag {
    pressed_at: f32,
    length_at_press: f32,
}

impl SplitDivider {
    /// Builds the divider: the line, and around it the room the pointer takes it by, with a
    /// handle that shows under the pointer.
    pub(crate) fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let axis_class = self.axis.class();
        let handle = mount.element().class(CLASSES.split_handle).class(axis_class).id();
        let grip = mount
            .element()
            .class(CLASSES.split_grip)
            .class(axis_class)
            .child(handle)
            .id();
        let value = self.clone();
        let number = mount.runtime().memo(move |runtime| value.number(runtime));
        let label = self.label.clone();
        let orientation = self.axis.divider_orientation();
        let mut drag = None;
        mount
            .element()
            .class(CLASSES.split_divider)
            .class(axis_class)
            .focusable(Focusability::TabStop)
            .accessible_role(AccessibleRole::Splitter)
            .accessible_name(label)
            .accessible_orientation(orientation)
            .accessible_number(number)
            .accessible_actions(
                AccessibleActions::INCREMENT
                    .union(AccessibleActions::DECREMENT)
                    .union(AccessibleActions::SET_VALUE),
            )
            .on_event(move |runtime, event| match event {
                ElementEvent::Pointer(pointer) => self.pointer_event(runtime, pointer, &mut drag),
                ElementEvent::Key(key) => self.key_event(runtime, key),
                ElementEvent::Accessibility(action) => self.accessibility_action(runtime, *action),
                ElementEvent::TextChanged(_) | ElementEvent::Drag(_) | ElementEvent::DragEnded(_) => {
                    Propagation::Continue
                }
            })
            .child(grip)
            .id()
    }

    /// How long the pane is shown: a folded pane takes no room.
    fn shown_length(&self, runtime: &mut Runtime<Surfaces>) -> f32 {
        if self.is_folded(runtime) {
            0.0
        } else {
            runtime.get(self.length)
        }
    }

    fn is_folded(&self, runtime: &mut Runtime<Surfaces>) -> bool {
        self.is_collapsed.is_some_and(|is_collapsed| runtime.get(is_collapsed))
    }

    /// The length of the pane as an assistive technology reads it.
    fn number(&self, runtime: &mut Runtime<Surfaces>) -> NumericValue {
        let shortest = if self.is_collapsed.is_some() {
            0.0
        } else {
            *self.limits.start()
        };
        NumericValue {
            value: f64::from(self.shown_length(runtime)),
            min: f64::from(shortest),
            max: f64::from(*self.limits.end()),
            step: f64::from(STEP),
        }
    }

    fn pointer_event(
        &self,
        runtime: &mut Runtime<Surfaces>,
        pointer: &PointerEvent,
        drag: &mut Option<Drag>,
    ) -> Propagation {
        match pointer.kind {
            PointerEventKind::Pressed {
                button: PointerButton::Primary,
                ..
            } => {
                *drag = Some(Drag {
                    pressed_at: self.axis.along(pointer.position),
                    length_at_press: self.shown_length(runtime),
                });
                Propagation::Stop
            }
            PointerEventKind::Moved => {
                let Some(drag) = *drag else {
                    return Propagation::Continue;
                };
                let travelled = self.axis.along(pointer.position) - drag.pressed_at;
                let dragged_to = if self.pane_lies_before {
                    drag.length_at_press + travelled
                } else {
                    drag.length_at_press - travelled
                };
                self.set_length(runtime, dragged_to);
                Propagation::Stop
            }
            PointerEventKind::Released {
                button: PointerButton::Primary,
            } => {
                *drag = None;
                Propagation::Stop
            }
            _ => Propagation::Continue,
        }
    }

    /// The arrow keys of the axis move the divider a step the way they point; Return and Space
    /// fold and unfold a pane that can fold.
    fn key_event(&self, runtime: &mut Runtime<Surfaces>, key: &KeyEvent) -> Propagation {
        if key.phase != KeyPhase::Pressed || !key.modifiers.is_empty() {
            return Propagation::Continue;
        }
        if matches!(key.key, Key::Enter | Key::Space) {
            let Some(is_collapsed) = self.is_collapsed else {
                return Propagation::Continue;
            };
            if !key.is_repeat {
                runtime.update(is_collapsed, |is_collapsed| *is_collapsed = !*is_collapsed);
            }
            return Propagation::Stop;
        }
        let lengthens = if key.key == self.axis.forward_arrow() {
            self.pane_lies_before
        } else if key.key == self.axis.backward_arrow() {
            !self.pane_lies_before
        } else {
            return Propagation::Continue;
        };
        self.step(runtime, lengthens);
        Propagation::Stop
    }

    fn accessibility_action(&self, runtime: &mut Runtime<Surfaces>, action: AccessibilityAction<'_>) -> Propagation {
        match action {
            AccessibilityAction::Increment => self.step(runtime, true),
            AccessibilityAction::Decrement => self.step(runtime, false),
            #[expect(
                clippy::cast_possible_truncation,
                reason = "lengths on a surface are single precision"
            )]
            AccessibilityAction::SetNumber(length) => self.set_length(runtime, length as f32),
            AccessibilityAction::Click | AccessibilityAction::SetText(_) => return Propagation::Continue,
        }
        Propagation::Stop
    }

    /// Lengthens or shortens the pane by a step. A folded pane unfolds at its shortest length,
    /// and a pane that can fold does so when shortened at that length.
    fn step(&self, runtime: &mut Runtime<Surfaces>, lengthens: bool) {
        let shortest = *self.limits.start();
        if self.is_folded(runtime) {
            if lengthens {
                self.set_length(runtime, shortest);
            }
            return;
        }
        let now = runtime.get(self.length);
        if lengthens {
            self.set_length(runtime, now + STEP);
        } else if now > shortest || self.is_collapsed.is_none() {
            self.set_length(runtime, (now - STEP).max(shortest));
        } else {
            self.set_length(runtime, 0.0);
        }
    }

    /// Makes the pane `asked` long, within its limits. A pane that can fold does so when asked
    /// to be less than half of its shortest length, and unfolds otherwise.
    fn set_length(&self, runtime: &mut Runtime<Surfaces>, asked: f32) {
        let (shortest, longest) = (*self.limits.start(), *self.limits.end());
        if let Some(is_collapsed) = self.is_collapsed {
            let folds = asked < shortest / 2.0;
            if runtime.get(is_collapsed) != folds {
                runtime.set(is_collapsed, folds);
            }
            if folds {
                return;
            }
        }
        let length = asked.clamp(shortest, longest);
        if (runtime.get(self.length) - length).abs() > f32::EPSILON {
            runtime.set(self.length, length);
        }
    }
}
