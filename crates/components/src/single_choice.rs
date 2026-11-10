use std::rc::Rc;

use morok_core::{ElementId, Focusability, Key, KeyEvent, KeyPhase, Propagation};
use morok_reactive::{Memo, Runtime, Signal};

use crate::activation::{ActivationKeys, handle_activation};
use crate::view::{ElementEvent, Mount, Property, Surfaces};

/// The options of a group of which one is chosen at a time, as radio buttons and the segments
/// of a segmented control are: the option whose value the signal holds is chosen. Activating an
/// option chooses it, and an arrow key chooses the option after or before the chosen one, or
/// the focused one while none is chosen. The Tab key stops on the chosen option alone, or on
/// the first while none is, and focus inside the group follows the choice. The choice moves at
/// once and focus with the next flush, so keys that come faster than frames move it on.
pub(crate) struct SingleChoice<Value> {
    selected: Signal<Value>,
    values: Rc<[Value]>,
    /// The place of the chosen option among the options; none when the signal holds a value
    /// of no option.
    chosen: Memo<Option<usize>>,
    is_disabled: Property<bool>,
}

impl<Value: Clone + PartialEq + 'static> SingleChoice<Value> {
    /// # Panics
    ///
    /// When there are no options.
    pub(crate) fn new(
        runtime: &mut Runtime<Surfaces>,
        selected: Signal<Value>,
        values: Vec<Value>,
        is_disabled: Property<bool>,
    ) -> Self {
        assert!(!values.is_empty(), "a group to choose from has options");
        let values: Rc<[Value]> = values.into();
        let options = Rc::clone(&values);
        let chosen = runtime
            .memo(move |runtime| runtime.with(selected, |selected| options.iter().position(|value| value == selected)));
        Self {
            selected,
            values,
            chosen,
            is_disabled,
        }
    }

    /// Whether the option at `position` is the chosen one.
    pub(crate) fn is_chosen(&self, runtime: &mut Runtime<Surfaces>, position: usize) -> Memo<bool> {
        let chosen = self.chosen;
        runtime.memo(move |runtime| runtime.get(chosen) == Some(position))
    }

    /// How the option at `position` takes focus: the Tab key stops on one option of the group.
    pub(crate) fn focusability(&self, runtime: &mut Runtime<Surfaces>, position: usize) -> Memo<Focusability> {
        let chosen = self.chosen;
        runtime.memo(move |runtime| {
            if runtime.get(chosen).unwrap_or(0) == position {
                Focusability::TabStop
            } else {
                Focusability::Focusable
            }
        })
    }

    /// What the option at `position` does with the events it hears of.
    pub(crate) fn handler(
        &self,
        position: usize,
    ) -> impl FnMut(&mut Runtime<Surfaces>, &ElementEvent<'_>) -> Propagation + 'static {
        let selected = self.selected;
        let values = Rc::clone(&self.values);
        let is_disabled = self.is_disabled.clone();
        move |runtime, event| {
            if let ElementEvent::Key(key) = event
                && let Some(direction) = ArrowDirection::of(key)
            {
                if !is_disabled.get(runtime) {
                    let count = values.len();
                    let chosen = runtime.with(selected, |selected| values.iter().position(|value| value == selected));
                    let from = chosen.unwrap_or(position);
                    let next = match direction {
                        ArrowDirection::Next => (from + 1) % count,
                        ArrowDirection::Previous => (from + count - 1) % count,
                    };
                    runtime.set(selected, values[next].clone());
                }
                return Propagation::Stop;
            }
            handle_activation(runtime, event, ActivationKeys::Space, &is_disabled, &mut |runtime| {
                runtime.set(selected, values[position].clone());
            })
        }
    }

    /// Moves focus to the chosen option whenever the choice changes while focus is on one of
    /// `options`, the elements of the options in their order.
    pub(crate) fn focus_follows_choice(&self, mount: &mut Mount<'_>, options: Vec<ElementId>) {
        let chosen = self.chosen;
        mount.effect(move |runtime, tree| {
            let Some(chosen) = runtime.get(chosen) else {
                return;
            };
            if tree.focused().is_some_and(|focused| options.contains(&focused)) {
                tree.focus(options[chosen]);
            }
        });
    }
}

/// Where an arrow key moves the choice: the arrows that point right and down choose the next
/// option and the others the one before, around the ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArrowDirection {
    Next,
    Previous,
}

impl ArrowDirection {
    fn of(key: &KeyEvent) -> Option<Self> {
        if key.phase != KeyPhase::Pressed || !key.modifiers.is_empty() {
            return None;
        }
        match key.key {
            Key::ArrowRight | Key::ArrowDown => Some(Self::Next),
            Key::ArrowLeft | Key::ArrowUp => Some(Self::Previous),
            _ => None,
        }
    }
}
