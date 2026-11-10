use std::fmt::{Debug, Formatter, Result};

use morok_core::ElementId;
use morok_core::accesskit::Role as AccessibleRole;
use morok_reactive::Signal;
use morok_style::States;

use crate::check_row::CheckRow;
use crate::single_choice::SingleChoice;
use crate::styles::CLASSES;
use crate::view::{Mount, Property};

/// Radio buttons, one under another, of which one is chosen: the one whose value the signal
/// holds. A click on a radio button or on its label, Space while it has focus and an
/// assistive technology choose it; the arrow keys choose the next or the previous one. The Tab
/// key stops on the chosen radio button alone.
pub struct RadioGroup<Value> {
    selected: Signal<Value>,
    options: Vec<(Value, Property<String>)>,
    label: Property<String>,
    is_disabled: Property<bool>,
}

impl<Value: Clone + PartialEq + 'static> RadioGroup<Value> {
    /// A group with no radio buttons yet.
    pub fn new(selected: Signal<Value>) -> Self {
        Self {
            selected,
            options: Vec::new(),
            label: Property::Fixed(String::new()),
            is_disabled: Property::Fixed(false),
        }
    }

    /// Adds a radio button that chooses `value`, after the ones added before.
    #[must_use]
    pub fn option(mut self, value: Value, label: impl Into<Property<String>>) -> Self {
        self.options.push((value, label.into()));
        self
    }

    /// What an assistive technology calls the group.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// The radio buttons of a disabled group take no focus and do not change the choice.
    #[must_use]
    pub fn disabled(mut self, is_disabled: impl Into<Property<bool>>) -> Self {
        self.is_disabled = is_disabled.into();
        self
    }

    /// Builds the group: a column of rows, each a round box with the dot of the chosen one and
    /// the text of the label.
    ///
    /// # Panics
    ///
    /// When the group has no radio buttons.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self {
            selected,
            options,
            label,
            is_disabled,
        } = self;
        let values = options.iter().map(|(value, _)| value.clone()).collect();
        let choice = SingleChoice::new(mount.runtime(), selected, values, is_disabled.clone());
        let mut radios = Vec::with_capacity(options.len());
        for (position, (_, option_label)) in options.into_iter().enumerate() {
            let dot = mount.element().class(CLASSES.radio_dot).id();
            let mut radio_box = mount.element();
            radio_box.class(CLASSES.check_box).child(dot);
            let radio_box = radio_box.id();
            let is_chosen = choice.is_chosen(mount.runtime(), position);
            let focusability = choice.focusability(mount.runtime(), position);
            let radio = CheckRow {
                role: AccessibleRole::RadioButton,
                variant: Some(CLASSES.radio),
                label: option_label,
                is_checked: is_chosen.into(),
                is_disabled: is_disabled.clone(),
                focusability: focusability.into(),
            }
            .mount(mount, radio_box, choice.handler(position));
            radios.push(radio);
        }
        choice.focus_follows_choice(mount, radios.clone());
        let mut group = mount.element();
        group
            .class(CLASSES.radio_group)
            .state(States::DISABLED, is_disabled)
            .accessible_role(AccessibleRole::RadioGroup)
            .accessible_name(label);
        for radio in radios {
            group.child(radio);
        }
        group.id()
    }
}

impl<Value: Debug> Debug for RadioGroup<Value> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("RadioGroup")
            .field("selected", &self.selected)
            .field("options", &self.options)
            .field("label", &self.label)
            .field("is_disabled", &self.is_disabled)
            .finish()
    }
}
