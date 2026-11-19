use std::cell::RefCell;
use std::rc::Rc;

use crate::view::{Action, Property};

/// An answer to a modal panel: a button of the panel, and a key that presses it.
#[derive(Clone)]
pub(crate) struct ModalAction {
    pub(crate) label: Property<String>,
    /// Shared by the button and the key, and by every opening of the panel.
    pub(crate) run: Rc<RefCell<Action>>,
}
