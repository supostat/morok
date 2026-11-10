use std::any::Any;
use std::fmt;

use crate::running_computation::RunningComputation;
use crate::runtime::Runtime;

/// Computes the value of a memo into its slot and returns whether the value changed.
pub(crate) type MemoCompute = Box<dyn FnMut(&mut Runtime, &mut Option<Box<dyn Any>>) -> bool>;
pub(crate) type EffectRun = Box<dyn FnMut(&mut Runtime)>;

const MEMO_READS_ITSELF: &str = "a memo does not read its own value while computing it";

pub(crate) enum NodeContent {
    Scope,
    Signal(Box<dyn Any>),
    Memo {
        value: Option<Box<dyn Any>>,
        compute: Option<MemoCompute>,
    },
    Effect(Option<EffectRun>),
}

impl NodeContent {
    pub(crate) fn is_effect(&self) -> bool {
        matches!(self, Self::Effect(_))
    }

    /// # Panics
    ///
    /// When the node holds no value, or is a memo that is computing its value right now.
    pub(crate) fn value(&self) -> &dyn Any {
        match self {
            Self::Signal(value) => value.as_ref(),
            Self::Memo { value, .. } => value.as_deref().expect(MEMO_READS_ITSELF),
            Self::Scope | Self::Effect(_) => panic!("only a signal or a memo holds a value"),
        }
    }

    /// # Panics
    ///
    /// When the node is not a signal.
    pub(crate) fn signal_value_mut(&mut self) -> &mut dyn Any {
        match self {
            Self::Signal(value) => value.as_mut(),
            Self::Scope | Self::Memo { .. } | Self::Effect(_) => panic!("only a signal is written"),
        }
    }

    /// # Panics
    ///
    /// When the node computes nothing, or is already running.
    pub(crate) fn start_running(&mut self) -> RunningComputation {
        match self {
            Self::Memo { value, compute } => RunningComputation::Memo {
                compute: compute.take().expect(MEMO_READS_ITSELF),
                value: value.take(),
            },
            Self::Effect(run) => RunningComputation::Effect(run.take().expect("an effect does not run inside itself")),
            Self::Scope | Self::Signal(_) => panic!("only a memo or an effect runs"),
        }
    }
}

impl fmt::Debug for NodeContent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Scope => "Scope",
            Self::Signal(_) => "Signal",
            Self::Memo { .. } => "Memo",
            Self::Effect(_) => "Effect",
        })
    }
}
