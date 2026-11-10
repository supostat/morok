use morok_reactive::Runtime;

use crate::view::surfaces::Surfaces;

/// What a component does when something is asked of it — a press, a shortcut: it is handed the
/// signal graph to write to, and the effects of the next flush change the tree.
pub type Action = Box<dyn FnMut(&mut Runtime<Surfaces>)>;
