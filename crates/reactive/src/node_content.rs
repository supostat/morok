use std::any::Any;
use std::fmt;

use crate::runtime::Runtime;

/// Computes the value of a memo into its slot and returns whether the value changed.
pub(crate) type MemoCompute<World> = Box<dyn FnMut(&mut Runtime<World>, &mut Option<Box<dyn Any>>) -> bool>;
pub(crate) type EffectRun<World> = Box<dyn FnMut(&mut Runtime<World>, &mut World)>;

const MEMO_READS_ITSELF: &str = "a memo does not read its own value while computing it";

pub(crate) enum NodeContent<World> {
    Scope,
    Signal(Box<dyn Any>),
    Memo {
        value: Option<Box<dyn Any>>,
        compute: Option<MemoCompute<World>>,
    },
    Effect(Option<EffectRun<World>>),
}

impl<World> NodeContent<World> {
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

    /// Hands over what a memo computes with and the value it holds; the node takes both back
    /// when the computation ends.
    ///
    /// # Panics
    ///
    /// When the node is not a memo, or is a memo that is computing its value right now.
    pub(crate) fn take_memo(&mut self) -> (MemoCompute<World>, Option<Box<dyn Any>>) {
        match self {
            Self::Memo { value, compute } => (compute.take().expect(MEMO_READS_ITSELF), value.take()),
            Self::Scope | Self::Signal(_) | Self::Effect(_) => panic!("only a memo computes a value"),
        }
    }

    /// Hands over what an effect runs; the node takes it back when the run ends.
    ///
    /// # Panics
    ///
    /// When the node is not an effect, or is an effect that is running right now.
    pub(crate) fn take_effect(&mut self) -> EffectRun<World> {
        match self {
            Self::Effect(run) => run.take().expect("an effect does not run inside itself"),
            Self::Scope | Self::Signal(_) | Self::Memo { .. } => panic!("only an effect runs in a flush"),
        }
    }
}

impl<World> fmt::Debug for NodeContent<World> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Scope => "Scope",
            Self::Signal(_) => "Signal",
            Self::Memo { .. } => "Memo",
            Self::Effect(_) => "Effect",
        })
    }
}
