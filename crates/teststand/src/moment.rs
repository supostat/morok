use std::time::Duration;

use crate::scene::Scene;

/// When a case is drawn.
#[derive(Clone, Copy, Debug)]
pub enum Moment {
    /// Once the first frame has run: a case without motion.
    AtRest,
    /// `after` the first frame. Keyframe animations run from the first frame; `change` is made
    /// to the tree once the first frame styled it, and the transitions it starts run from then.
    InMotion { change: fn(&mut Scene), after: Duration },
}
