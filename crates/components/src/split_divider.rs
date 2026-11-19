use std::ops::RangeInclusive;

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{
    AccessibilityAction, AccessibleActions, ElementId, Focusability, Key, KeyEvent, KeyPhase, NumericValue,
    PointerButton, PointerEvent, PointerEventKind, Propagation,
};
use morok_reactive::{Runtime, Signal};

use crate::styles::CLASSES;
use crate::view::{ElementEvent, Mount, Property, Surfaces};

/// How far an arrow key or an assistive technology moves a divider at once.
const STEP: f32 = 16.0;

/// The divider between two panes of a split view: a line the pointer drags and the arrow keys
/// move. It sets the width of the pane it belongs to, within the limits of that pane, and
/// folds a pane that can fold.
#[derive(Clone, Debug)]
pub(crate) struct SplitDivider {
    /// What an assistive technology calls the divider.
    pub(crate) label: Property<String>,
    pub(crate) width: Signal<f32>,
    pub(crate) limits: RangeInclusive<f32>,
    pub(crate) is_collapsed: Option<Signal<bool>>,
    /// Whether the pane lies before the divider, so that moving the divider right widens it.
    pub(crate) pane_lies_before: bool,
}

/// A drag of a divider: where the pointer went down and how wide the pane was then.
#[derive(Clone, Copy, Debug)]
struct Drag {
    pressed_at: f32,
    width_at_press: f32,
}

impl SplitDivider {
    /// Builds the divider: the line, and around it the room the pointer takes it by, with a
    /// handle that shows under the pointer.
    pub(crate) fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let handle = mount.element().class(CLASSES.split_handle).id();
        let grip = mount.element().class(CLASSES.split_grip).child(handle).id();
        let value = self.clone();
        let number = mount.runtime().memo(move |runtime| value.number(runtime));
        let label = self.label.clone();
        let mut drag = None;
        mount
            .element()
            .class(CLASSES.split_divider)
            .focusable(Focusability::TabStop)
            .accessible_role(AccessibleRole::Splitter)
            .accessible_name(label)
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
                ElementEvent::TextChanged(_) => Propagation::Continue,
            })
            .child(grip)
            .id()
    }

    /// How wide the pane is shown: a folded pane takes no room.
    fn shown_width(&self, runtime: &mut Runtime<Surfaces>) -> f32 {
        if self.is_folded(runtime) {
            0.0
        } else {
            runtime.get(self.width)
        }
    }

    fn is_folded(&self, runtime: &mut Runtime<Surfaces>) -> bool {
        self.is_collapsed.is_some_and(|is_collapsed| runtime.get(is_collapsed))
    }

    /// The width of the pane as an assistive technology reads it.
    fn number(&self, runtime: &mut Runtime<Surfaces>) -> NumericValue {
        let narrowest = if self.is_collapsed.is_some() {
            0.0
        } else {
            *self.limits.start()
        };
        NumericValue {
            value: f64::from(self.shown_width(runtime)),
            min: f64::from(narrowest),
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
                    pressed_at: pointer.position.x,
                    width_at_press: self.shown_width(runtime),
                });
                Propagation::Stop
            }
            PointerEventKind::Moved => {
                let Some(drag) = *drag else {
                    return Propagation::Continue;
                };
                let travelled = pointer.position.x - drag.pressed_at;
                let dragged_to = if self.pane_lies_before {
                    drag.width_at_press + travelled
                } else {
                    drag.width_at_press - travelled
                };
                self.set_width(runtime, dragged_to);
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

    /// The arrow keys move the divider a step the way they point; Return and Space fold and
    /// unfold a pane that can fold.
    fn key_event(&self, runtime: &mut Runtime<Surfaces>, key: &KeyEvent) -> Propagation {
        if key.phase != KeyPhase::Pressed || !key.modifiers.is_empty() {
            return Propagation::Continue;
        }
        let widens = match key.key {
            Key::ArrowRight => self.pane_lies_before,
            Key::ArrowLeft => !self.pane_lies_before,
            Key::Enter | Key::Space => {
                let Some(is_collapsed) = self.is_collapsed else {
                    return Propagation::Continue;
                };
                if !key.is_repeat {
                    runtime.update(is_collapsed, |is_collapsed| *is_collapsed = !*is_collapsed);
                }
                return Propagation::Stop;
            }
            _ => return Propagation::Continue,
        };
        self.step(runtime, widens);
        Propagation::Stop
    }

    fn accessibility_action(&self, runtime: &mut Runtime<Surfaces>, action: AccessibilityAction<'_>) -> Propagation {
        match action {
            AccessibilityAction::Increment => self.step(runtime, true),
            AccessibilityAction::Decrement => self.step(runtime, false),
            #[expect(
                clippy::cast_possible_truncation,
                reason = "widths on a surface are single precision"
            )]
            AccessibilityAction::SetNumber(width) => self.set_width(runtime, width as f32),
            AccessibilityAction::Click | AccessibilityAction::SetText(_) => return Propagation::Continue,
        }
        Propagation::Stop
    }

    /// Widens or narrows the pane by a step. A folded pane unfolds at its narrowest width,
    /// and a pane that can fold does so when narrowed at that width.
    fn step(&self, runtime: &mut Runtime<Surfaces>, widens: bool) {
        let narrowest = *self.limits.start();
        if self.is_folded(runtime) {
            if widens {
                self.set_width(runtime, narrowest);
            }
            return;
        }
        let now = runtime.get(self.width);
        if widens {
            self.set_width(runtime, now + STEP);
        } else if now > narrowest || self.is_collapsed.is_none() {
            self.set_width(runtime, (now - STEP).max(narrowest));
        } else {
            self.set_width(runtime, 0.0);
        }
    }

    /// Makes the pane `asked` wide, within its limits. A pane that can fold does so when
    /// asked to be less than half of its narrowest width, and unfolds otherwise.
    fn set_width(&self, runtime: &mut Runtime<Surfaces>, asked: f32) {
        let (narrowest, widest) = (*self.limits.start(), *self.limits.end());
        if let Some(is_collapsed) = self.is_collapsed {
            let folds = asked < narrowest / 2.0;
            if runtime.get(is_collapsed) != folds {
                runtime.set(is_collapsed, folds);
            }
            if folds {
                return;
            }
        }
        let width = asked.clamp(narrowest, widest);
        if (runtime.get(self.width) - width).abs() > f32::EPSILON {
            runtime.set(self.width, width);
        }
    }
}
