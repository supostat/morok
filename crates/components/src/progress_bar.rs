use std::fmt::{Debug, Formatter, Result};

use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{ElementId, NumericValue};
use morok_style::Declaration;

use crate::fraction_of_box::fraction_of_box;
use crate::styles::CLASSES;
use crate::view::{Mount, Property};

/// A bar that shows how far a task has come: filled as far as the task is done, or, while
/// that is not known, crossed again and again by a stripe.
pub struct ProgressBar {
    /// From zero to one; none while it is not known.
    done: Option<Property<f64>>,
    label: Property<String>,
}

impl ProgressBar {
    /// A bar filled as far as `done` says the task is done, from zero to one.
    pub fn determinate(done: impl Into<Property<f64>>) -> Self {
        Self {
            done: Some(done.into()),
            label: Property::Fixed(String::new()),
        }
    }

    /// A bar for a task under way that does not know how far it has come.
    pub fn indeterminate() -> Self {
        Self {
            done: None,
            label: Property::Fixed(String::new()),
        }
    }

    /// What an assistive technology calls the bar: the task it follows.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// Builds the bar: a track that holds the fill. An assistive technology reads how far
    /// the task has come in percent.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self { done, label } = self;
        let runtime = mount.runtime();
        let done = done.map(|done| runtime.memo(move |runtime| done.get(runtime).clamp(0.0, 1.0)));
        let fill_width =
            done.map(|done| runtime.memo(move |runtime| Declaration::Width(fraction_of_box(runtime.get(done)))));
        let percent = done.map(|done| {
            runtime.memo(move |runtime| NumericValue {
                value: runtime.get(done) * 100.0,
                min: 0.0,
                max: 100.0,
                step: 1.0,
            })
        });
        let mut fill = mount.element();
        fill.class(CLASSES.progress_fill);
        if let Some(fill_width) = fill_width {
            fill.inline(fill_width);
        }
        let fill = fill.id();
        let mut bar = mount.element();
        bar.class(CLASSES.progress)
            .accessible_role(AccessibleRole::ProgressIndicator)
            .accessible_name(label);
        match percent {
            Some(percent) => {
                bar.accessible_number(percent);
            }
            None => {
                bar.class(CLASSES.indeterminate);
            }
        }
        bar.child(fill).id()
    }
}

impl Debug for ProgressBar {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("ProgressBar")
            .field("done", &self.done)
            .field("label", &self.label)
            .finish()
    }
}
