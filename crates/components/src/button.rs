use std::fmt::{Debug, Formatter, Result};

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{AccessibleActions, ElementId, Focusability};
use morok_reactive::Runtime;
use morok_style::States;

use crate::activation::{ActivationKeys, handle_activation};
use crate::styles::CLASSES;
use crate::view::{Action, Mount, Property, Surfaces};

/// A push button: a label that is pressed by a click, by Space or Return while it has focus,
/// and by an assistive technology. A disabled button is pressed by none of them.
pub struct Button {
    label: Property<String>,
    is_primary: bool,
    is_disabled: Property<bool>,
    on_press: Action,
}

impl Button {
    /// A button of the default look that is named by its label and does nothing when pressed.
    pub fn new(label: impl Into<Property<String>>) -> Self {
        Self {
            label: label.into(),
            is_primary: false,
            is_disabled: Property::Fixed(false),
            on_press: Box::new(|_| {}),
        }
    }

    /// Makes the button the one of its group that stands out: the main action.
    #[must_use]
    pub fn primary(mut self) -> Self {
        self.is_primary = true;
        self
    }

    #[must_use]
    pub fn disabled(mut self, is_disabled: impl Into<Property<bool>>) -> Self {
        self.is_disabled = is_disabled.into();
        self
    }

    /// Names what a press of the button does.
    #[must_use]
    pub fn on_press(mut self, handler: impl FnMut(&mut Runtime<Surfaces>) + 'static) -> Self {
        self.on_press = Box::new(handler);
        self
    }

    /// Builds the button: a box that holds the text of the label.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self {
            label,
            is_primary,
            is_disabled,
            mut on_press,
        } = self;
        let label_text = mount.text(label.clone()).id();
        let press_is_disabled = is_disabled.clone();
        let mut button = mount.element();
        button.class(CLASSES.button);
        if is_primary {
            button.class(CLASSES.primary);
        }
        button
            .state(States::DISABLED, is_disabled)
            .focusable(Focusability::TabStop)
            .accessible_role(AccessibleRole::Button)
            .accessible_name(label)
            .accessible_actions(AccessibleActions::CLICK)
            .on_event(move |runtime, event| {
                handle_activation(
                    runtime,
                    event,
                    ActivationKeys::SpaceAndReturn,
                    &press_is_disabled,
                    &mut on_press,
                )
            })
            .child(label_text)
            .id()
    }
}

impl Debug for Button {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("Button")
            .field("label", &self.label)
            .field("is_primary", &self.is_primary)
            .field("is_disabled", &self.is_disabled)
            .finish_non_exhaustive()
    }
}
