use morok_core::ElementId;
use morok_core::accesskit::Role as AccessibleRole;

use crate::styles::CLASSES;
use crate::view::{Mount, Property};

/// A ring that turns while something is under way that does not say how far it has come.
#[derive(Debug)]
pub struct Spinner {
    label: Property<String>,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            label: Property::Fixed(String::new()),
        }
    }

    /// What an assistive technology calls the spinner: what is under way.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// Builds the spinner: one box whose border turns.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        mount
            .element()
            .class(CLASSES.spinner)
            .accessible_role(AccessibleRole::ProgressIndicator)
            .accessible_name(self.label)
            .id()
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}
