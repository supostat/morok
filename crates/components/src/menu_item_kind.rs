use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use crate::menu_item::MenuItem;
use crate::view::Action;

/// What a line of a menu is.
pub(crate) enum MenuItemKind {
    /// A line the user chooses; the menu closes then.
    Command { on_choose: RefCell<Action> },
    /// A line that opens a menu of its own beside it.
    Submenu { items: Rc<[MenuItem]> },
    /// A thin line between groups of items.
    Separator,
}

impl Debug for MenuItemKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self {
            Self::Command { .. } => formatter.write_str("Command"),
            Self::Submenu { items } => formatter.debug_tuple("Submenu").field(items).finish(),
            Self::Separator => formatter.write_str("Separator"),
        }
    }
}
