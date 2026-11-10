use std::any::Any;
use std::collections::VecDeque;
use std::mem;

use crate::effect::Effect;
use crate::flush_cycle::FlushCycle;
use crate::graph::Graph;
use crate::memo::Memo;
use crate::node::Node;
use crate::node_content::NodeContent;
use crate::node_id::NodeId;
use crate::node_state::NodeState;
use crate::scope::Scope;
use crate::signal::Signal;
use crate::source::Source;
use crate::tracking::Tracking;

/// How many times one effect may run in one flush before the flush is reported as a cycle.
const RUNS_PER_FLUSH: u32 = 100;
const VALUE_TYPE: &str = "a handle reads the type its node was created with";

/// The signal graph of one thread.
///
/// A write only marks what depends on the signal and queues the effects among it; [`flush`]
/// runs the queue. Memos are computed when they are read. A memo or an effect depends on
/// exactly what its last run read, and owns what that run created: the next run disposes it.
///
/// `World` is what the effects act on besides the graph: [`flush`] hands it to every effect it
/// runs. A graph whose effects touch nothing but signals has the world `()`.
///
/// A handle that outlived the scope that owns its node panics when it is used.
///
/// [`flush`]: Runtime::flush
#[derive(Debug)]
pub struct Runtime<World = ()> {
    graph: Graph<World>,
    tracking: Tracking,
    runs_started: u64,
    flushes_started: u64,
    is_flushing: bool,
    queued_effects: VecDeque<NodeId>,
    /// Scratch of `mark_subscribers`, kept for its capacity.
    newly_stale: Vec<NodeId>,
}

impl<World> Default for Runtime<World> {
    fn default() -> Self {
        Self {
            graph: Graph::default(),
            tracking: Tracking::default(),
            runs_started: 0,
            flushes_started: 0,
            is_flushing: false,
            queued_effects: VecDeque::new(),
            newly_stale: Vec::new(),
        }
    }
}

impl<World> Runtime<World> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn signal<T: 'static>(&mut self, value: T) -> Signal<T> {
        Signal::new(self.insert(NodeContent::Signal(Box::new(value)), NodeState::Clean))
    }

    /// A value computed by `compute` on its first read and again after a value it read changed.
    /// A new value equal to the old one wakes nothing that depends on the memo.
    pub fn memo<T: PartialEq + 'static>(&mut self, mut compute: impl FnMut(&mut Self) -> T + 'static) -> Memo<T> {
        let compute = move |runtime: &mut Self, slot: &mut Option<Box<dyn Any>>| {
            let new_value = compute(runtime);
            let Some(value) = slot else {
                *slot = Some(Box::new(new_value));
                return true;
            };
            let value = value.downcast_mut::<T>().expect(VALUE_TYPE);
            if *value == new_value {
                return false;
            }
            *value = new_value;
            true
        };
        let content = NodeContent::Memo {
            value: None,
            compute: Some(Box::new(compute)),
        };
        Memo::new(self.insert(content, NodeState::Dirty))
    }

    /// A function that runs in the next flush, and again in the flush after a value it read
    /// changed. Every run is given the world of the flush that runs it.
    pub fn effect(&mut self, run: impl FnMut(&mut Self, &mut World) + 'static) -> Effect {
        let id = self.insert(NodeContent::Effect(Some(Box::new(run))), NodeState::Dirty);
        self.graph.node_mut(id).is_queued = true;
        self.queued_effects.push_back(id);
        Effect { id }
    }

    /// A scope that owns everything `build` creates. It belongs to the memo, effect or scope
    /// that runs right now, and is disposed with it.
    pub fn scope(&mut self, build: impl FnOnce(&mut Self)) -> Scope {
        self.scope_owned_by(self.tracking.owner, build)
    }

    /// A scope that owns everything `build` creates and belongs to `owner`, whatever runs right
    /// now: an effect creates with it what is to outlive its own run.
    pub fn scope_in(&mut self, owner: Scope, build: impl FnOnce(&mut Self)) -> Scope {
        self.scope_owned_by(Some(owner.id), build)
    }

    /// # Panics
    ///
    /// When a memo or an effect of the scope is running.
    pub fn dispose(&mut self, scope: Scope) {
        self.graph.dispose(scope.id);
    }

    pub fn get<T: Clone + 'static>(&mut self, source: impl Into<Source<T>>) -> T {
        self.with(source, T::clone)
    }

    /// Reads the value in place. The memo or effect that runs right now comes to depend on it.
    ///
    /// # Panics
    ///
    /// When a memo reads its own value while computing it.
    pub fn with<T: 'static, R>(&mut self, source: impl Into<Source<T>>, read: impl FnOnce(&T) -> R) -> R {
        let id = source.into().id;
        // The reader subscribes after the source is up to date: a memo that computes a new
        // value for this read wakes the readers of its old value, not this one.
        self.refresh_source(id);
        self.track(id);
        read(
            self.graph
                .node(id)
                .content
                .value()
                .downcast_ref::<T>()
                .expect(VALUE_TYPE),
        )
    }

    pub fn set<T: 'static>(&mut self, signal: Signal<T>, value: T) {
        self.update(signal, |current| *current = value);
    }

    /// Changes the value in place. What depends on the signal becomes stale whether or not the
    /// value differs.
    pub fn update<T: 'static>(&mut self, signal: Signal<T>, change: impl FnOnce(&mut T)) {
        let value = self.graph.node_mut(signal.id).content.signal_value_mut();
        change(value.downcast_mut::<T>().expect(VALUE_TYPE));
        self.mark_subscribers(signal.id);
    }

    /// Runs `read` so that the memo or effect that runs right now does not come to depend on
    /// what it reads.
    pub fn untracked<R>(&mut self, read: impl FnOnce(&mut Self) -> R) -> R {
        let outer_observer = self.tracking.observer.take();
        let result = read(self);
        self.tracking.observer = outer_observer;
        result
    }

    /// Whether a flush has an effect to run: one that was created, or woken by a write, since
    /// the last flush.
    pub fn has_queued_effects(&self) -> bool {
        !self.queued_effects.is_empty()
    }

    /// Runs the stale effects on `world` until none is left. However many signals were written
    /// since the last flush, an effect runs once and sees only values that agree with each
    /// other. An effect that an effect of this flush wakes or creates runs in this flush too.
    ///
    /// # Errors
    ///
    /// When an effect keeps being woken by the writes of this flush. That effect stays stale
    /// until one of its sources is written again; the effects still queued run in the next flush.
    ///
    /// # Panics
    ///
    /// When called from inside an effect.
    pub fn flush(&mut self, world: &mut World) -> Result<(), FlushCycle> {
        assert!(!self.is_flushing, "flush is not called from inside an effect");
        self.is_flushing = true;
        self.flushes_started += 1;
        let outcome = self.run_queued_effects(world);
        self.is_flushing = false;
        outcome
    }

    fn run_queued_effects(&mut self, world: &mut World) -> Result<(), FlushCycle> {
        while let Some(id) = self.queued_effects.pop_front() {
            let flush = self.flushes_started;
            let Some(effect) = self.graph.find_mut(id) else {
                continue;
            };
            effect.is_queued = false;
            if effect.counted_flush != flush {
                effect.counted_flush = flush;
                effect.runs_in_flush = 0;
            }
            if effect.runs_in_flush == RUNS_PER_FLUSH {
                return Err(FlushCycle { effect: Effect { id } });
            }
            effect.runs_in_flush += 1;
            self.refresh_owners_first(id, world);
        }
        Ok(())
    }

    fn insert(&mut self, content: NodeContent<World>, state: NodeState) -> NodeId {
        self.graph.insert(Node::new(content, state), self.tracking.owner)
    }

    fn scope_owned_by(&mut self, owner: Option<NodeId>, build: impl FnOnce(&mut Self)) -> Scope {
        let id = self
            .graph
            .insert(Node::new(NodeContent::Scope, NodeState::Clean), owner);
        let outer_owner = self.tracking.owner.replace(id);
        build(self);
        self.tracking.owner = outer_owner;
        Scope { id }
    }

    fn track(&mut self, source: NodeId) {
        let Some(observer) = self.tracking.observer else {
            return;
        };
        let run = self.tracking.run;
        let node = self.graph.node_mut(source);
        if node.read_in_run != run {
            node.read_in_run = run;
            self.graph.link(observer, source);
        }
    }

    /// A stale owner disposes what it owns when it runs, so the owners of a node are brought
    /// up to date before the node itself, the outermost first.
    fn refresh_owners_first(&mut self, id: NodeId, world: &mut World) {
        if let Some(owner) = self.graph.node(id).owner {
            self.refresh_owners_first(owner, world);
        }
        if self.graph.contains(id) {
            self.refresh_in_flush(id, world);
        }
    }

    /// Brings a memo or an effect up to date with its sources; a scope is always up to date.
    fn refresh_in_flush(&mut self, id: NodeId, world: &mut World) {
        if !self.is_behind_its_sources(id) {
            return;
        }
        if self.graph.node(id).content.is_effect() {
            self.run_effect(id, world);
        } else {
            self.run_memo(id);
        }
    }

    /// Brings a memo up to date with its sources; a signal is always up to date.
    fn refresh_source(&mut self, id: NodeId) {
        if self.is_behind_its_sources(id) {
            self.run_memo(id);
        }
    }

    /// Whether the node has to run again. A node that only had to check its sources, and found
    /// none of them changed, is clean from here on.
    fn is_behind_its_sources(&mut self, id: NodeId) -> bool {
        if self.graph.node(id).state == NodeState::Check {
            self.refresh_sources_until_dirty(id);
        }
        let node = self.graph.node_mut(id);
        if node.state == NodeState::Dirty {
            return true;
        }
        node.state = NodeState::Clean;
        false
    }

    /// A source that computes a new value marks the node dirty; the remaining sources are
    /// then left to the run of the node.
    fn refresh_sources_until_dirty(&mut self, id: NodeId) {
        let mut position = 0;
        while let Some(edge) = self.graph.node(id).sources.get(position).copied() {
            self.refresh_source(edge.node);
            let node = self.graph.node(id);
            if node.state == NodeState::Dirty {
                return;
            }
            let source_is_still_here = node.sources.get(position) == Some(&edge);
            if source_is_still_here {
                position += 1;
            }
        }
    }

    fn run_memo(&mut self, id: NodeId) {
        let outer_tracking = self.begin_run(id);
        let (mut compute, mut value) = self.graph.node_mut(id).content.take_memo();
        let value_changed = compute(self, &mut value);
        let content = NodeContent::Memo {
            value,
            compute: Some(compute),
        };
        self.end_run(id, content, outer_tracking);
        if value_changed {
            self.mark_subscribers(id);
        }
    }

    fn run_effect(&mut self, id: NodeId, world: &mut World) {
        let outer_tracking = self.begin_run(id);
        let mut run = self.graph.node_mut(id).content.take_effect();
        run(self, world);
        self.end_run(id, NodeContent::Effect(Some(run)), outer_tracking);
    }

    /// Disposes what the last run of the node created and forgets what it read: from here to
    /// the end of the run, what is read and created is accounted to the node. Returns what the
    /// code around the run is accounted to.
    fn begin_run(&mut self, id: NodeId) -> Tracking {
        self.graph.dispose_children(id);
        self.graph.unlink_sources(id);
        self.runs_started += 1;
        let node = self.graph.node_mut(id);
        // Clean before the run: a write of the run that wakes the node itself is not lost.
        node.state = NodeState::Clean;
        node.is_running = true;
        mem::replace(
            &mut self.tracking,
            Tracking {
                observer: Some(id),
                owner: Some(id),
                run: self.runs_started,
            },
        )
    }

    fn end_run(&mut self, id: NodeId, content: NodeContent<World>, outer_tracking: Tracking) {
        self.tracking = outer_tracking;
        let node = self.graph.node_mut(id);
        node.content = content;
        node.is_running = false;
    }

    /// The value of `source` changed: what reads it is dirty, and everything further down has
    /// to check its own sources. The effects among them are queued.
    fn mark_subscribers(&mut self, source: NodeId) {
        self.mark_subscribers_as(source, NodeState::Dirty);
        while let Some(stale) = self.newly_stale.pop() {
            self.mark_subscribers_as(stale, NodeState::Check);
        }
    }

    fn mark_subscribers_as(&mut self, source: NodeId, state: NodeState) {
        let mut position = 0;
        while let Some(edge) = self.graph.node(source).subscribers.get(position).copied() {
            let subscriber = self.graph.node_mut(edge.node);
            // What depends on an already stale node was marked when that node became stale.
            if subscriber.state == NodeState::Clean {
                self.newly_stale.push(edge.node);
            }
            subscriber.state = subscriber.state.max(state);
            if subscriber.content.is_effect() && !subscriber.is_queued {
                subscriber.is_queued = true;
                self.queued_effects.push_back(edge.node);
            }
            position += 1;
        }
    }
}
