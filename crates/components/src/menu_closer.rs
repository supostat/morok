use std::rc::Rc;

use morok_reactive::Runtime;

use crate::view::Surfaces;

/// Closes a menu: writes the signal that keeps it open.
pub(crate) type MenuCloser = Rc<dyn Fn(&mut Runtime<Surfaces>)>;
