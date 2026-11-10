/// How far a memo or an effect is behind its sources, ordered from fresh to stale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum NodeState {
    Clean,
    /// A source further up was written: the node is stale only if one of its own sources
    /// turns out to have changed.
    Check,
    Dirty,
}
