use std::rc::Rc;

use morok_core::ElementId;

use crate::view::Mount;

/// What an application puts inside a panel of the kit: built anew every time the panel opens.
pub(crate) type PanelContent = Rc<dyn Fn(&mut Mount<'_>) -> ElementId>;
