use std::fmt::{Debug, Formatter, Result};

use morok_core::ElementId;
use morok_reactive::{Runtime, Signal};

use crate::text_field_parts::{FieldKind, FieldParts};
use crate::view::{Mount, Property, Surfaces};

/// One line of plain text the user edits, in a box that shows when it has focus and when
/// what it holds is not valid. What the user types is written to the signal it edits, and a
/// value written to the signal is shown.
pub struct TextField {
    parts: FieldParts,
}

impl TextField {
    pub fn new(value: Signal<String>) -> Self {
        Self {
            parts: FieldParts::new(value),
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

    /// Shows that what the field holds is not valid.
    #[must_use]
    pub fn invalid(mut self, is_invalid: impl Into<Property<bool>>) -> Self {
        self.parts.is_invalid = is_invalid.into();
        self
    }

    /// Names what Return does in the field.
    #[must_use]
    pub fn on_submit(mut self, handler: impl FnMut(&mut Runtime<Surfaces>) + 'static) -> Self {
        self.parts.on_submit = Some(Box::new(handler));
        self
    }

    /// Builds the field: a box that holds the edited line.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        self.parts.mount(FieldKind::Line, mount)
    }
}

impl Debug for TextField {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("TextField")
            .field("placeholder", &self.parts.placeholder)
            .field("is_disabled", &self.parts.is_disabled)
            .field("is_invalid", &self.parts.is_invalid)
            .finish_non_exhaustive()
    }
}
