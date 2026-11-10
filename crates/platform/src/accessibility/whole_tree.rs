use std::cell::RefCell;

use morok_core::accesskit::{ActivationHandler, TreeUpdate};

use crate::surface_content::SurfaceContent;

/// Gives the adapter the whole accessibility tree of the content the first time an assistive
/// technology asks the surface anything.
pub(crate) struct WholeTree<'a> {
    pub(crate) content: &'a RefCell<dyn SurfaceContent>,
}

impl ActivationHandler for WholeTree<'_> {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(self.content.borrow_mut().accessibility_tree())
    }
}
