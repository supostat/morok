use std::mem;

use accesskit::{Node, NodeId, TreeId, TreeInfo, TreeUpdate};
use morok_style::States;

use crate::accessibility::node_facts::NodeFacts;
use crate::accessibility::semantics::Semantics;
use crate::editing::text_runs::TextRuns;
use crate::element::Element;
use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::geometry::transform::Transform;
use crate::input::focusability::Focusability;
use crate::paint::group_transform::local_transform;
use crate::role::Role;

/// Works out a tree update: the node of every rendered element that has one, an element
/// without a node handing its children up to the nearest element with one.
///
/// An update holds a node only when the node is new to the tree or differs from what the
/// last update said of it; a whole tree holds every node.
#[derive(Debug)]
pub(crate) struct AccessibilityPass<'a> {
    pub(crate) elements: &'a mut ElementArena,
    pub(crate) root: ElementId,
    pub(crate) focused: Option<ElementId>,
    /// Device pixels to a CSS pixel on the surface.
    pub(crate) scale: f32,
    /// The number of this update; the update before it has the number one less.
    pub(crate) update: u64,
    pub(crate) holds_whole_tree: bool,
    pub(crate) nodes: Vec<(NodeId, Node)>,
}

impl AccessibilityPass<'_> {
    pub(crate) fn run(mut self) -> TreeUpdate {
        let mut above_the_root = Vec::new();
        self.visit(self.root, Transform::IDENTITY, &mut above_the_root);
        let root = NodeId::from(self.root);
        TreeUpdate {
            focus: self.focus_node(),
            tree: self.holds_whole_tree.then(|| TreeInfo::new(root)),
            tree_id: TreeId::ROOT,
            nodes: self.nodes,
        }
    }

    /// Adds the node of the element, or the nodes of what is inside it, to `siblings`.
    /// `around` takes a point of the element around this one to the surface.
    fn visit(&mut self, id: ElementId, around: Transform, siblings: &mut Vec<NodeId>) {
        let element = &self.elements[id];
        let is_shown = element.is_rendered;
        if !is_shown && id != self.root {
            return;
        }
        let on_surface = local_transform(element).then(&around);
        if element.semantics.is_none() {
            self.visit_children(id, on_surface, siblings);
            return;
        }
        siblings.push(NodeId::from(id));
        let mut children = mem::take(&mut self.semantics_mut(id).children_scratch);
        children.clear();
        if is_shown {
            self.visit_children(id, on_surface, &mut children);
        }
        let facts = self.facts(id, &on_surface);
        let element = &self.elements[id];
        let semantics = element
            .semantics
            .as_deref()
            .expect("the element was just seen to have a node");
        let published = &semantics.published;
        let is_as_published =
            published.update + 1 == self.update && published.facts == facts && published.children == children;
        if self.holds_whole_tree || !is_as_published {
            let text = match element.editable.as_deref() {
                Some(editable) => Some(editable.editor.text()),
                None => (element.role == Role::Text).then_some(element.text.as_str()),
            };
            let mut node = semantics.node(&facts, text, &children);
            if let Some(editable) = element.editable.as_deref().filter(|_| is_shown) {
                let runs = TextRuns {
                    element: id,
                    editable,
                    border_box: element.border_box,
                    on_surface: &on_surface,
                };
                runs.describe(&mut node, &mut self.nodes);
            }
            self.nodes.push((NodeId::from(id), node));
        }
        let update = self.update;
        let semantics = self.semantics_mut(id);
        semantics.published.update = update;
        semantics.published.facts = facts;
        mem::swap(&mut semantics.published.children, &mut children);
        semantics.children_scratch = children;
    }

    fn visit_children(&mut self, id: ElementId, on_surface: Transform, siblings: &mut Vec<NodeId>) {
        for position in 0..self.elements[id].children.len() {
            let child = self.elements[id].children[position];
            self.visit(child, on_surface, siblings);
        }
    }

    fn semantics_mut(&mut self, id: ElementId) -> &mut Semantics {
        self.elements[id]
            .semantics
            .as_deref_mut()
            .expect("the element was just seen to have a node")
    }

    fn facts(&self, id: ElementId, on_surface: &Transform) -> NodeFacts {
        let element = &self.elements[id];
        let semantics = element
            .semantics
            .as_deref()
            .expect("the element was just seen to have a node");
        let is_disabled = element.states.contains(States::DISABLED);
        NodeFacts {
            revision: semantics.revision(),
            bounds: on_surface.bounds_of(&element.border_box),
            is_disabled,
            is_checked: element.states.contains(States::CHECKED),
            takes_focus: element.focusability != Focusability::NotFocusable && !is_disabled,
            scale: (id == self.root).then_some(self.scale),
        }
    }

    /// The node that has focus: that of the focused element, or of the nearest element around
    /// it that this update holds a node of. With no focus the root has it.
    fn focus_node(&self) -> NodeId {
        let mut current = self.focused;
        while let Some(id) = current {
            let element = &self.elements[id];
            if self.holds_node_of(element) {
                return NodeId::from(id);
            }
            current = element.parent;
        }
        NodeId::from(self.root)
    }

    fn holds_node_of(&self, element: &Element) -> bool {
        element
            .semantics
            .as_deref()
            .is_some_and(|semantics| semantics.published.update == self.update)
    }
}
