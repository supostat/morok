use std::fmt::{Debug, Formatter, Result};

use morok_core::ElementId;

use crate::view::action::Action;

/// What a shortcut of an element does when its chord is pressed.
pub(crate) struct ShortcutHandler {
    /// The element the shortcut answers in; the shortcut goes when the element is removed.
    pub(crate) element: ElementId,
    pub(crate) run: Action,
}

impl Debug for ShortcutHandler {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("ShortcutHandler")
            .field("element", &self.element)
            .finish_non_exhaustive()
    }
}
