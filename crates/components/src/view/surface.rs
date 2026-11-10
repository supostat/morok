use morok_core::accesskit::ActionRequest;
use morok_core::{ElementId, ElementTree, KeyEvent, PointerInput};
use morok_reactive::{Runtime, Scope};
use morok_style::{StyleSheet, Theme};

use crate::view::child_fragments::ChildFragments;
use crate::view::event_dispatch::EventDispatch;
use crate::view::event_handlers::EventHandlers;
use crate::view::surfaces::Surfaces;

/// One surface as the components see it: its element tree and what they registered for its
/// elements. The host of the surface gives it the input of the surface and runs the stages of
/// its tree.
#[derive(Debug)]
pub struct Surface {
    pub(crate) tree: ElementTree,
    /// Owns everything mounted on the surface.
    pub(crate) scope: Scope,
    pub(crate) event_handlers: EventHandlers,
    pub(crate) child_fragments: ChildFragments,
    /// Scratch space for the elements of a subtree that is removed.
    removed_elements: Vec<ElementId>,
}

impl Surface {
    pub(crate) fn new(style_sheet: StyleSheet, theme: Theme, scope: Scope) -> Self {
        Self {
            tree: ElementTree::new(style_sheet, theme),
            scope,
            event_handlers: EventHandlers::default(),
            child_fragments: ChildFragments::default(),
            removed_elements: Vec::new(),
        }
    }

    pub fn tree(&self) -> &ElementTree {
        &self.tree
    }

    /// The tree itself, for the stages of a frame and for whoever drives its elements directly.
    pub fn tree_mut(&mut self) -> &mut ElementTree {
        &mut self.tree
    }

    /// Takes what the pointer did on the surface and lets the components hear of it.
    pub fn pointer_input(&mut self, runtime: &mut Runtime<Surfaces>, input: &PointerInput) {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.pointer_input(input, &mut dispatch);
    }

    /// Works out again what a resting pointer is over once a frame moved the boxes under it:
    /// called at the start of every frame.
    pub fn refresh_hover(&mut self, runtime: &mut Runtime<Surfaces>) {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.refresh_hover(&mut dispatch);
    }

    /// Takes a key that went down or up. Returns whether a component, a shortcut or the focus
    /// of the tree used it.
    pub fn key_input(&mut self, runtime: &mut Runtime<Surfaces>, event: &KeyEvent) -> bool {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.key_input(event, &mut dispatch)
    }

    /// Takes what an assistive technology asks of a node of the accessibility tree.
    pub fn accessibility_action(&mut self, runtime: &mut Runtime<Surfaces>, request: &ActionRequest) {
        let mut dispatch = EventDispatch {
            handlers: &mut self.event_handlers,
            runtime,
        };
        self.tree.accessibility_action(request, &mut dispatch);
    }

    /// Removes the element with everything inside it from the tree, and with them what the
    /// components registered for them. The scope that owns their effects is disposed first.
    pub(crate) fn remove_subtree(&mut self, root: ElementId) {
        self.removed_elements.clear();
        self.removed_elements.push(root);
        let mut visited = 0;
        while let Some(element) = self.removed_elements.get(visited).copied() {
            self.removed_elements.extend(self.tree.children(element));
            visited += 1;
        }
        for element in &self.removed_elements {
            self.event_handlers.forget_element(*element);
            self.child_fragments.forget_parent(*element);
        }
        self.tree.remove(root);
        self.event_handlers.forget_shortcuts_of_removed_elements(&self.tree);
    }
}
