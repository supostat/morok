use std::fmt::{Debug, Formatter, Result};
use std::ops::RangeInclusive;

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{
    AccessibilityAction, AccessibleActions, ElementId, Focusability, Key, KeyEvent, KeyPhase, PointerButton,
    PointerEvent, PointerEventKind, Propagation,
};
use morok_reactive::Signal;
use morok_style::{Declaration, Side, States};

use crate::fraction_of_box::fraction_of_box;
use crate::slider_scale::SliderScale;
use crate::styles::CLASSES;
use crate::view::{ElementEvent, Mount, Property};

/// A slider: a thumb on a rail that picks a number of a range, the part of the rail before the
/// thumb filled. The pointer drags the thumb or sets it down where it presses the rail; the
/// arrow keys move it a step, Page Up and Page Down a tenth of the range, Home and End to the
/// ends; an assistive technology moves it a step or sets the number. It shows the signal it is
/// given and writes what the user makes of it back to the signal.
pub struct Slider {
    value: Signal<f64>,
    scale: SliderScale,
    label: Property<String>,
    is_disabled: Property<bool>,
}

impl Slider {
    /// A slider from zero to one in hundredths.
    pub fn new(value: Signal<f64>) -> Self {
        Self {
            value,
            scale: SliderScale::new(0.0..=1.0, 0.01),
            label: Property::Fixed(String::new()),
            is_disabled: Property::Fixed(false),
        }
    }

    /// The numbers the slider picks from: the range, in steps of `step` from its start.
    ///
    /// # Panics
    ///
    /// When the range holds one number or none, or the step does not move forward.
    #[must_use]
    pub fn range(mut self, range: RangeInclusive<f64>, step: f64) -> Self {
        self.scale = SliderScale::new(range, step);
        self
    }

    /// What an assistive technology calls the slider.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// A disabled slider takes no focus and does not move.
    #[must_use]
    pub fn disabled(mut self, is_disabled: impl Into<Property<bool>>) -> Self {
        self.is_disabled = is_disabled.into();
        self
    }

    /// Builds the slider: a box that holds the rail, the fill and the thumb.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self {
            value,
            scale,
            label,
            is_disabled,
        } = self;
        let runtime = mount.runtime();
        let fraction = runtime.memo(move |runtime| scale.fraction_of(runtime.get(value)));
        let fill_width = runtime.memo(move |runtime| Declaration::Width(fraction_of_box(runtime.get(fraction))));
        let thumb_place =
            runtime.memo(move |runtime| Declaration::Inset(Side::Left, fraction_of_box(runtime.get(fraction))));
        let number = runtime.memo(move |runtime| scale.numeric_value(runtime.get(value)));
        let rail = mount.element().class(CLASSES.slider_rail).id();
        let fill = mount.element().class(CLASSES.slider_fill).inline(fill_width).id();
        let thumb = mount.element().class(CLASSES.slider_thumb).inline(thumb_place).id();
        let move_is_disabled = is_disabled.clone();
        let mut is_dragged = false;
        mount
            .element()
            .class(CLASSES.slider)
            .state(States::DISABLED, is_disabled)
            .focusable(Focusability::TabStop)
            .accessible_role(AccessibleRole::Slider)
            .accessible_name(label)
            .accessible_number(number)
            .accessible_actions(
                AccessibleActions::INCREMENT
                    .union(AccessibleActions::DECREMENT)
                    .union(AccessibleActions::SET_VALUE),
            )
            .on_event(move |runtime, event| {
                if move_is_disabled.get(runtime) {
                    return Propagation::Continue;
                }
                let now = runtime.get(value);
                let moved_to = match event {
                    ElementEvent::Pointer(pointer) => match pointer_drag(pointer, &mut is_dragged) {
                        Drag::Moves => value_under_pointer(scale, pointer),
                        Drag::Ends => return Propagation::Stop,
                        Drag::None => return Propagation::Continue,
                    },
                    ElementEvent::Key(key) => match value_after_key(scale, key, now) {
                        Some(moved_to) => Some(moved_to),
                        None => return Propagation::Continue,
                    },
                    ElementEvent::Accessibility(action) => match value_after_action(scale, *action, now) {
                        Some(moved_to) => Some(moved_to),
                        None => return Propagation::Continue,
                    },
                    ElementEvent::TextChanged(_) => return Propagation::Continue,
                };
                if let Some(moved_to) = moved_to {
                    runtime.set(value, moved_to);
                }
                Propagation::Stop
            })
            .child(rail)
            .child(fill)
            .child(thumb)
            .id()
    }
}

impl Debug for Slider {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("Slider")
            .field("value", &self.value)
            .field("scale", &self.scale)
            .field("label", &self.label)
            .field("is_disabled", &self.is_disabled)
            .finish()
    }
}

/// What a pointer event does to the drag of the thumb.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Drag {
    /// The thumb goes where the pointer is: the primary button went down on the slider, or
    /// the pointer moved while the button is down.
    Moves,
    /// The primary button came up and the thumb stays where it is.
    Ends,
    None,
}

/// Follows the drag through the event: the slider hears of the moves of the pointer while the
/// button is down wherever the pointer goes.
fn pointer_drag(pointer: &PointerEvent, is_dragged: &mut bool) -> Drag {
    match pointer.kind {
        PointerEventKind::Pressed {
            button: PointerButton::Primary,
            ..
        } => {
            *is_dragged = true;
            Drag::Moves
        }
        PointerEventKind::Moved if *is_dragged => Drag::Moves,
        PointerEventKind::Released {
            button: PointerButton::Primary,
        } if *is_dragged => {
            *is_dragged = false;
            Drag::Ends
        }
        _ => Drag::None,
    }
}

/// The value at the place of the pointer along the slider; none for a slider with no width,
/// which has no place to take it from.
fn value_under_pointer(scale: SliderScale, pointer: &PointerEvent) -> Option<f64> {
    let slider = pointer.element_box;
    (slider.width > 0.0).then(|| scale.value_at(f64::from((pointer.position.x - slider.x) / slider.width)))
}

fn value_after_key(scale: SliderScale, key: &KeyEvent, now: f64) -> Option<f64> {
    if key.phase != KeyPhase::Pressed || !key.modifiers.is_empty() {
        return None;
    }
    match key.key {
        Key::ArrowRight | Key::ArrowUp => Some(scale.stepped(now, 1.0)),
        Key::ArrowLeft | Key::ArrowDown => Some(scale.stepped(now, -1.0)),
        Key::PageUp => Some(scale.paged(now, 1.0)),
        Key::PageDown => Some(scale.paged(now, -1.0)),
        Key::Home => Some(scale.lowest()),
        Key::End => Some(scale.highest()),
        _ => None,
    }
}

fn value_after_action(scale: SliderScale, action: AccessibilityAction<'_>, now: f64) -> Option<f64> {
    match action {
        AccessibilityAction::Increment => Some(scale.stepped(now, 1.0)),
        AccessibilityAction::Decrement => Some(scale.stepped(now, -1.0)),
        AccessibilityAction::SetNumber(number) => Some(scale.snapped(number)),
        AccessibilityAction::Click | AccessibilityAction::SetText(_) => None,
    }
}
