use std::fmt::{Debug, Formatter, Result};

use morok_core::ElementId;

use crate::view::{Mount, Property};

/// What stands in a toolbar, in its order.
pub(crate) enum ToolbarPart {
    /// A control the application builds.
    Item(Box<dyn FnOnce(&mut Mount<'_>) -> ElementId>),
    /// The title of the window, in bold.
    Title(Property<String>),
    /// Room that takes what the other parts leave and pushes what follows to the far end.
    Spacer,
}

impl Debug for ToolbarPart {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self {
            Self::Item(_) => formatter.write_str("Item"),
            Self::Title(title) => formatter.debug_tuple("Title").field(title).finish(),
            Self::Spacer => formatter.write_str("Spacer"),
        }
    }
}
