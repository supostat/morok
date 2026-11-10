use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{AccessibleActions, ElementId, Focusability, Propagation};
use morok_reactive::Runtime;
use morok_style::{Class, States};

use crate::styles::CLASSES;
use crate::view::{ElementEvent, Mount, Property, Surfaces};

/// What a check box, a switch and a radio button are built from: a row of the class `check`
/// that holds the part that shows the state, its indicator, and the text of the label. The row
/// is the node of the control, takes the focus and hears of the input; the indicator is checked
/// when the row is, so that the stylesheet draws it from either.
pub(crate) struct CheckRow {
    pub(crate) role: AccessibleRole,
    /// The class that tells the row of this control from the rows of the others, if any.
    pub(crate) variant: Option<Class>,
    pub(crate) label: Property<String>,
    pub(crate) is_checked: Property<bool>,
    pub(crate) is_disabled: Property<bool>,
    pub(crate) focusability: Property<Focusability>,
}

impl CheckRow {
    /// Builds the row around `indicator`; `handler` hears of the events of the row.
    pub(crate) fn mount(
        self,
        mount: &mut Mount<'_>,
        indicator: ElementId,
        handler: impl FnMut(&mut Runtime<Surfaces>, &ElementEvent<'_>) -> Propagation + 'static,
    ) -> ElementId {
        let Self {
            role,
            variant,
            label,
            is_checked,
            is_disabled,
            focusability,
        } = self;
        mount.build(indicator).state(States::CHECKED, is_checked.clone());
        let label_text = mount.text(label.clone()).id();
        let mut row = mount.element();
        row.class(CLASSES.check);
        if let Some(variant) = variant {
            row.class(variant);
        }
        row.state(States::CHECKED, is_checked)
            .state(States::DISABLED, is_disabled)
            .focusable(focusability)
            .accessible_role(role)
            .accessible_name(label)
            .accessible_actions(AccessibleActions::CLICK)
            .on_event(handler)
            .child(indicator)
            .child(label_text)
            .id()
    }
}
