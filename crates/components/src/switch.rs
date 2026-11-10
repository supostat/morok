use std::fmt::{Debug, Formatter, Result};

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{ElementId, Focusability};
use morok_reactive::Signal;

use crate::activation::{ActivationKeys, handle_activation};
use crate::check_row::CheckRow;
use crate::styles::CLASSES;
use crate::view::{Mount, Property};

/// A switch with its label: a track whose knob slides to the side that says whether a setting
/// is on. A click on either, Space while it has focus and an assistive technology flip it. It
/// shows the signal it is given and writes what the user makes of it back to the signal.
pub struct Switch {
    is_on: Signal<bool>,
    label: Property<String>,
    is_disabled: Property<bool>,
}

impl Switch {
    pub fn new(is_on: Signal<bool>, label: impl Into<Property<String>>) -> Self {
        Self {
            is_on,
            label: label.into(),
            is_disabled: Property::Fixed(false),
        }
    }

    /// A disabled switch takes no focus and does not change.
    #[must_use]
    pub fn disabled(mut self, is_disabled: impl Into<Property<bool>>) -> Self {
        self.is_disabled = is_disabled.into();
        self
    }

    /// Builds the switch: a row that holds the track with its knob, and the text of the label.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self {
            is_on,
            label,
            is_disabled,
        } = self;
        let knob = mount.element().class(CLASSES.switch_knob).id();
        let mut track = mount.element();
        track.class(CLASSES.switch).child(knob);
        let track = track.id();
        let flip_is_disabled = is_disabled.clone();
        CheckRow {
            role: AccessibleRole::Switch,
            variant: None,
            label,
            is_checked: is_on.into(),
            is_disabled,
            focusability: Property::Fixed(Focusability::TabStop),
        }
        .mount(mount, track, move |runtime, event| {
            handle_activation(
                runtime,
                event,
                ActivationKeys::Space,
                &flip_is_disabled,
                &mut |runtime| runtime.update(is_on, |is_on| *is_on = !*is_on),
            )
        })
    }
}

impl Debug for Switch {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("Switch")
            .field("is_on", &self.is_on)
            .field("label", &self.label)
            .field("is_disabled", &self.is_disabled)
            .finish()
    }
}
