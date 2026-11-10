use std::fmt::{Debug, Formatter, Result};

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{AccessibleActions, ElementId};
use morok_reactive::Signal;
use morok_style::States;

use crate::single_choice::SingleChoice;
use crate::styles::CLASSES;
use crate::view::{Mount, Property};

/// Segments side by side in a sunken strip, of which one is chosen and raised: the one whose
/// value the signal holds. A click, Space while a segment has focus and an assistive
/// technology choose a segment; the arrow keys choose the next or the previous one. The Tab key
/// stops on the chosen segment alone.
pub struct SegmentedControl<Value> {
    selected: Signal<Value>,
    segments: Vec<(Value, Property<String>)>,
    label: Property<String>,
    is_disabled: Property<bool>,
}

impl<Value: Clone + PartialEq + 'static> SegmentedControl<Value> {
    /// A control with no segments yet.
    pub fn new(selected: Signal<Value>) -> Self {
        Self {
            selected,
            segments: Vec::new(),
            label: Property::Fixed(String::new()),
            is_disabled: Property::Fixed(false),
        }
    }

    /// Adds a segment that chooses `value`, after the ones added before.
    #[must_use]
    pub fn segment(mut self, value: Value, label: impl Into<Property<String>>) -> Self {
        self.segments.push((value, label.into()));
        self
    }

    /// What an assistive technology calls the control.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// The segments of a disabled control take no focus and do not change the choice.
    #[must_use]
    pub fn disabled(mut self, is_disabled: impl Into<Property<bool>>) -> Self {
        self.is_disabled = is_disabled.into();
        self
    }

    /// Builds the control: a strip that holds a box with the text of each segment.
    ///
    /// # Panics
    ///
    /// When the control has no segments.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self {
            selected,
            segments,
            label,
            is_disabled,
        } = self;
        let values = segments.iter().map(|(value, _)| value.clone()).collect();
        let choice = SingleChoice::new(mount.runtime(), selected, values, is_disabled.clone());
        let mut segment_elements = Vec::with_capacity(segments.len());
        for (position, (_, segment_label)) in segments.into_iter().enumerate() {
            let label_text = mount.text(segment_label.clone()).id();
            let is_chosen = choice.is_chosen(mount.runtime(), position);
            let focusability = choice.focusability(mount.runtime(), position);
            let segment = mount
                .element()
                .class(CLASSES.segment)
                .state(States::CHECKED, is_chosen)
                .state(States::DISABLED, is_disabled.clone())
                .focusable(focusability)
                .accessible_role(AccessibleRole::RadioButton)
                .accessible_name(segment_label)
                .accessible_actions(AccessibleActions::CLICK)
                .on_event(choice.handler(position))
                .child(label_text)
                .id();
            segment_elements.push(segment);
        }
        choice.focus_follows_choice(mount, segment_elements.clone());
        let mut control = mount.element();
        control
            .class(CLASSES.segmented)
            .state(States::DISABLED, is_disabled)
            .accessible_role(AccessibleRole::RadioGroup)
            .accessible_name(label);
        for segment in segment_elements {
            control.child(segment);
        }
        control.id()
    }
}

impl<Value: Debug> Debug for SegmentedControl<Value> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("SegmentedControl")
            .field("selected", &self.selected)
            .field("segments", &self.segments)
            .field("label", &self.label)
            .field("is_disabled", &self.is_disabled)
            .finish()
    }
}
