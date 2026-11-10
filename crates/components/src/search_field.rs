use std::fmt::{Debug, Formatter, Result};

use morok_core::ElementId;
use morok_reactive::{Runtime, Signal};

use crate::text_field_parts::{FieldKind, FieldParts};
use crate::view::{Mount, Property, Surfaces};

/// One line the user types what to search for into, in a rounded box with a magnifier. What
/// the user types is written to the signal it edits.
pub struct SearchField {
    parts: FieldParts,
}

impl SearchField {
    pub fn new(query: Signal<String>) -> Self {
        Self {
            parts: FieldParts::new(query),
        }
    }

    /// What the field shows while it is empty.
    #[must_use]
    pub fn placeholder(mut self, placeholder: impl Into<Property<String>>) -> Self {
        self.parts.placeholder = placeholder.into();
        self
    }

    /// What an assistive technology calls the field.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.parts.label = label.into();
        self
    }

    /// A disabled field takes no focus and so no input.
    #[must_use]
    pub fn disabled(mut self, is_disabled: impl Into<Property<bool>>) -> Self {
        self.parts.is_disabled = is_disabled.into();
        self
    }

    /// Names what Return does in the field.
    #[must_use]
    pub fn on_submit(mut self, handler: impl FnMut(&mut Runtime<Surfaces>) + 'static) -> Self {
        self.parts.on_submit = Some(Box::new(handler));
        self
    }

    /// Builds the field: a box that holds the magnifier and the edited line.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        self.parts.mount(FieldKind::Search, mount)
    }
}

impl Debug for SearchField {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("SearchField")
            .field("placeholder", &self.parts.placeholder)
            .field("is_disabled", &self.parts.is_disabled)
            .finish_non_exhaustive()
    }
}
