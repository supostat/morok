use std::fmt::{Debug, Formatter, Result};

use morok_core::ElementId;
use morok_core::Focusability;
use morok_core::accesskit::Role as AccessibleRole;
use morok_reactive::Signal;

use crate::activation::{ActivationKeys, handle_activation};
use crate::check_row::CheckRow;
use crate::kit_icon::KitIcon;
use crate::styles::CLASSES;
use crate::view::{Mount, Property};

/// A check box with its label: a click on either, Space while it has focus and an assistive
/// technology check and uncheck it. It shows the signal it is given and writes what the user
/// makes of it back to the signal.
pub struct Checkbox {
    is_checked: Signal<bool>,
    label: Property<String>,
    is_disabled: Property<bool>,
}

impl Checkbox {
    pub fn new(is_checked: Signal<bool>, label: impl Into<Property<String>>) -> Self {
        Self {
            is_checked,
            label: label.into(),
            is_disabled: Property::Fixed(false),
        }
    }

    /// A disabled check box takes no focus and does not change.
    #[must_use]
    pub fn disabled(mut self, is_disabled: impl Into<Property<bool>>) -> Self {
        self.is_disabled = is_disabled.into();
        self
    }

    /// Builds the check box: a row that holds a box with the mark of a checked box, and the
    /// text of the label.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self {
            is_checked,
            label,
            is_disabled,
        } = self;
        let mark = mount.kit_icon(KitIcon::Check).class(CLASSES.check_mark).id();
        let mut check_box = mount.element();
        check_box.class(CLASSES.check_box).child(mark);
        let check_box = check_box.id();
        let toggle_is_disabled = is_disabled.clone();
        CheckRow {
            role: AccessibleRole::CheckBox,
            variant: None,
            label,
            is_checked: is_checked.into(),
            is_disabled,
            focusability: Property::Fixed(Focusability::TabStop),
        }
        .mount(mount, check_box, move |runtime, event| {
            handle_activation(
                runtime,
                event,
                ActivationKeys::Space,
                &toggle_is_disabled,
                &mut |runtime| runtime.update(is_checked, |is_checked| *is_checked = !*is_checked),
            )
        })
    }
}

impl Debug for Checkbox {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("Checkbox")
            .field("is_checked", &self.is_checked)
            .field("label", &self.label)
            .field("is_disabled", &self.is_disabled)
            .finish()
    }
}
