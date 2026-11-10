use accesskit::{Action, Affine, Node, NodeId, Rect, Role, Toggled};

use crate::accessibility::accessible_actions::AccessibleActions;
use crate::accessibility::accessible_value::AccessibleValue;
use crate::accessibility::node_facts::NodeFacts;
use crate::accessibility::published_node::PublishedNode;
use crate::accessibility::stored_value::StoredValue;

/// What an element is to an assistive technology: the element has a node in the accessibility
/// tree for as long as it has this.
#[derive(Debug)]
pub(crate) struct Semantics {
    role: Role,
    name: String,
    value: StoredValue,
    actions: AccessibleActions,
    /// How many times any of the above, or the text of the element, was changed.
    revision: u64,
    pub(crate) published: PublishedNode,
    /// Scratch space for the children of the node while an update is worked out.
    pub(crate) children_scratch: Vec<NodeId>,
}

impl Semantics {
    pub(crate) fn new(role: Role) -> Self {
        Self {
            role,
            name: String::new(),
            value: StoredValue::None,
            actions: AccessibleActions::NONE,
            revision: 0,
            published: PublishedNode::default(),
            children_scratch: Vec::new(),
        }
    }

    pub(crate) fn role(&self) -> Role {
        self.role
    }

    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn actions(&self) -> AccessibleActions {
        self.actions
    }

    /// Each setter returns whether it changed anything.
    pub(crate) fn set_role(&mut self, role: Role) -> bool {
        let changes = self.role != role;
        self.role = role;
        self.note(changes)
    }

    pub(crate) fn set_name(&mut self, name: &str) -> bool {
        let changes = self.name != name;
        if changes {
            self.name.clear();
            self.name.push_str(name);
        }
        self.note(changes)
    }

    pub(crate) fn set_value(&mut self, value: AccessibleValue<'_>) -> bool {
        let changes = !self.value.is(value);
        if changes {
            self.value = StoredValue::from(value);
        }
        self.note(changes)
    }

    pub(crate) fn set_actions(&mut self, actions: AccessibleActions) -> bool {
        let changes = self.actions != actions;
        self.actions = actions;
        self.note(changes)
    }

    /// The text of the element changed: a text element gives its text as its value.
    pub(crate) fn text_changed(&mut self) {
        self.revision += 1;
    }

    fn note(&mut self, changes: bool) -> bool {
        if changes {
            self.revision += 1;
        }
        changes
    }

    /// Whether the checked state of the element is a part of what the role is.
    fn toggles(&self) -> bool {
        matches!(
            self.role,
            Role::CheckBox | Role::Switch | Role::RadioButton | Role::MenuItemCheckBox | Role::MenuItemRadio
        )
    }

    /// The node of the element; `text` is the text of a text element.
    pub(crate) fn node(&self, facts: &NodeFacts, text: Option<&str>, children: &[NodeId]) -> Node {
        let mut node = Node::new(self.role);
        if !self.name.is_empty() {
            node.set_label(self.name.as_str());
        }
        match (&self.value, text) {
            (StoredValue::Text(value), _) => node.set_value(value.as_str()),
            (StoredValue::Number(number), _) => {
                node.set_numeric_value(number.value);
                node.set_min_numeric_value(number.min);
                node.set_max_numeric_value(number.max);
                node.set_numeric_value_step(number.step);
            }
            (StoredValue::None, Some(text)) => node.set_value(text),
            (StoredValue::None, None) => {}
        }
        node.set_bounds(Rect {
            x0: f64::from(facts.bounds.x),
            y0: f64::from(facts.bounds.y),
            x1: f64::from(facts.bounds.right()),
            y1: f64::from(facts.bounds.bottom()),
        });
        if let Some(scale) = facts.scale {
            // Boxes are in CSS pixels and an adapter reads device pixels: the transform of
            // the root takes every box across.
            node.set_transform(Affine::scale(f64::from(scale)));
        }
        if facts.is_disabled {
            node.set_disabled();
        }
        if self.toggles() {
            node.set_toggled(Toggled::from(facts.is_checked));
        }
        if facts.takes_focus {
            node.add_action(Action::Focus);
        }
        for (offered, action) in [
            (AccessibleActions::CLICK, Action::Click),
            (AccessibleActions::INCREMENT, Action::Increment),
            (AccessibleActions::DECREMENT, Action::Decrement),
            (AccessibleActions::SET_VALUE, Action::SetValue),
        ] {
            if self.actions.contains(offered) {
                node.add_action(action);
            }
        }
        if !children.is_empty() {
            node.set_children(children);
        }
        node
    }
}
