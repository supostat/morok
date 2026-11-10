use std::cell::RefCell;
use std::rc::Rc;

use morok_core::accesskit::{ActionHandler, ActionRequest};

use crate::surface_content::SurfaceContent;

/// Hands what an assistive technology asks of a node to the content of the surface.
pub(crate) struct ContentActions {
    pub(crate) content: Rc<RefCell<dyn SurfaceContent>>,
}

impl ActionHandler for ContentActions {
    fn do_action(&mut self, request: ActionRequest) {
        self.content.borrow_mut().accessibility_action(&request);
    }
}
