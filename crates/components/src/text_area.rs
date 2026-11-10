use std::fmt::{Debug, Formatter, Result};

use morok_core::ElementId;
use morok_reactive::Signal;

use crate::text_field_parts::{FieldKind, FieldParts};
use crate::view::{Mount, Property};

/// Plain text the user edits in lines that wrap at the width of its box and scroll inside it.
/// What the user types is written to the signal it edits, and a value written to the signal
/// is shown.
pub struct TextArea {
    parts: FieldParts,
}

impl TextArea {
    pub fn new(value: Signal<String>) -> Self {
        Self {
            parts: FieldParts::new(value),
        }
    }

    /// What the area shows while it is empty.
    #[must_use]
    pub fn placeholder(mut self, placeholder: impl Into<Property<String>>) -> Self {
        self.parts.placeholder = placeholder.into();
        self
    }

    /// What an assistive technology calls the area.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.parts.label = label.into();
        self
    }

    /// A disabled area takes no focus and so no input.
    #[must_use]
    pub fn disabled(mut self, is_disabled: impl Into<Property<bool>>) -> Self {
        self.parts.is_disabled = is_disabled.into();
        self
    }

    /// Shows that what the area holds is not valid.
    #[must_use]
    pub fn invalid(mut self, is_invalid: impl Into<Property<bool>>) -> Self {
        self.parts.is_invalid = is_invalid.into();
        self
    }

    /// Builds the area: a box that holds the edited lines.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        self.parts.mount(FieldKind::Area, mount)
    }
}

impl Debug for TextArea {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("TextArea")
            .field("placeholder", &self.parts.placeholder)
            .field("is_disabled", &self.parts.is_disabled)
            .field("is_invalid", &self.parts.is_invalid)
            .finish_non_exhaustive()
    }
}
