use std::fmt::{Debug, Formatter, Result};

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{AccessibleActions, ElementId, Focusability};
use morok_reactive::{Runtime, Signal};
use morok_style::States;

use crate::activation::{ActivationKeys, handle_activation};
use crate::kit_icon::KitIcon;
use crate::styles::CLASSES;
use crate::view::{Action, Mount, Property, Surfaces};

/// A button that shows an icon in place of a label, as a toolbar does: pressed by a click, by
/// Space or Return while it has focus, and by an assistive technology, which calls it by its
/// label. A toggle stays pressed while its signal holds and a press flips it.
pub struct IconButton {
    icon: KitIcon,
    label: Property<String>,
    is_on: Option<Signal<bool>>,
    is_disabled: Property<bool>,
    on_press: Action,
}

impl IconButton {
    /// A button of the icon that does nothing when pressed; `label` names it, though it shows
    /// no text.
    pub fn new(icon: KitIcon, label: impl Into<Property<String>>) -> Self {
        Self {
            icon,
            label: label.into(),
            is_on: None,
            is_disabled: Property::Fixed(false),
            on_press: Box::new(|_| {}),
        }
    }

    /// Makes the button a toggle that stays pressed while `is_on` holds; a press flips it
    /// before the handler of the press runs.
    #[must_use]
    pub fn toggled(mut self, is_on: Signal<bool>) -> Self {
        self.is_on = Some(is_on);
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

    /// Builds the button: a square box that holds the icon.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self {
            icon,
            label,
            is_on,
            is_disabled,
            mut on_press,
        } = self;
        let icon = mount.kit_icon(icon).class(CLASSES.icon_button_icon).id();
        let press_is_disabled = is_disabled.clone();
        let mut button = mount.element();
        button
            .class(CLASSES.icon_button)
            .state(States::DISABLED, is_disabled)
            .focusable(Focusability::TabStop)
            .accessible_role(AccessibleRole::Button)
            .accessible_name(label)
            .accessible_actions(AccessibleActions::CLICK);
        if let Some(is_on) = is_on {
            button.state(States::CHECKED, is_on).accessible_toggle();
        }
        button
            .on_event(move |runtime, event| {
                handle_activation(
                    runtime,
                    event,
                    ActivationKeys::SpaceAndReturn,
                    &press_is_disabled,
                    &mut |runtime| {
                        if let Some(is_on) = is_on {
                            runtime.update(is_on, |is_on| *is_on = !*is_on);
                        }
                        on_press(runtime);
                    },
                )
            })
            .child(icon)
            .id()
    }
}

impl Debug for IconButton {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("IconButton")
            .field("icon", &self.icon)
            .field("label", &self.label)
            .field("is_on", &self.is_on)
            .field("is_disabled", &self.is_disabled)
            .finish_non_exhaustive()
    }
}
