use std::fmt::{Debug, Formatter, Result};

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{AccessibleActions, ElementId};
use morok_reactive::Signal;
use morok_style::States;

use crate::single_choice::SingleChoice;
use crate::styles::CLASSES;
use crate::view::{Mount, Property};

/// Tabs side by side over a line, of which one is selected and underlined: the one whose value
/// the signal holds. A click, Space while a tab has focus and an assistive technology select a
/// tab; the arrow keys select the next or the previous one. The Tab key stops on the selected
/// tab alone. What a tab shows is the application's: it follows the signal.
pub struct Tabs<Value> {
    selected: Signal<Value>,
    choices: Vec<(Value, Property<String>)>,
    label: Property<String>,
    is_disabled: Property<bool>,
}

impl<Value: Clone + PartialEq + 'static> Tabs<Value> {
    /// A strip with no tabs yet.
    pub fn new(selected: Signal<Value>) -> Self {
        Self {
            selected,
            choices: Vec::new(),
            label: Property::Fixed(String::new()),
            is_disabled: Property::Fixed(false),
        }
    }

    /// Adds a tab that selects `value`, after the ones added before.
    #[must_use]
    pub fn tab(mut self, value: Value, label: impl Into<Property<String>>) -> Self {
        self.choices.push((value, label.into()));
        self
    }

    /// What an assistive technology calls the strip.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// The tabs of a disabled strip take no focus and do not change the selection.
    #[must_use]
    pub fn disabled(mut self, is_disabled: impl Into<Property<bool>>) -> Self {
        self.is_disabled = is_disabled.into();
        self
    }

    /// Builds the strip: a row that holds a box with the text of each tab.
    ///
    /// # Panics
    ///
    /// When the strip has no tabs.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self {
            selected,
            choices,
            label,
            is_disabled,
        } = self;
        let values = choices.iter().map(|(value, _)| value.clone()).collect();
        let choice = SingleChoice::new(mount.runtime(), selected, values, is_disabled.clone());
        let mut tab_elements = Vec::with_capacity(choices.len());
        for (position, (_, tab_label)) in choices.into_iter().enumerate() {
            let label_text = mount.text(tab_label.clone()).id();
            let is_chosen = choice.is_chosen(mount.runtime(), position);
            let focusability = choice.focusability(mount.runtime(), position);
            let tab = mount
                .element()
                .class(CLASSES.tab)
                .state(States::CHECKED, is_chosen)
                .state(States::DISABLED, is_disabled.clone())
                .focusable(focusability)
                .accessible_role(AccessibleRole::Tab)
                .accessible_name(tab_label)
                .accessible_actions(AccessibleActions::CLICK)
                .on_event(choice.handler(position))
                .child(label_text)
                .id();
            tab_elements.push(tab);
        }
        choice.focus_follows_choice(mount, tab_elements.clone());
        let mut strip = mount.element();
        strip
            .class(CLASSES.tabs)
            .state(States::DISABLED, is_disabled)
            .accessible_role(AccessibleRole::TabList)
            .accessible_name(label);
        for tab in tab_elements {
            strip.child(tab);
        }
        strip.id()
    }
}

impl<Value: Debug> Debug for Tabs<Value> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("Tabs")
            .field("selected", &self.selected)
            .field("choices", &self.choices)
            .field("label", &self.label)
            .field("is_disabled", &self.is_disabled)
            .finish()
    }
}
