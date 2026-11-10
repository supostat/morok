use std::any::Any;

use crate::node_content::{EffectRun, MemoCompute};

/// What a node hands over while it runs: the runtime is free for the computation to read
/// and write through, and the node takes everything back afterwards.
pub(crate) enum RunningComputation {
    Memo {
        compute: MemoCompute,
        value: Option<Box<dyn Any>>,
    },
    Effect(EffectRun),
}
