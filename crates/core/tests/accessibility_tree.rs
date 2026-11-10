mod support;

use std::collections::HashMap;

use morok_core::accesskit::{
    Action, ActionData, ActionRequest, Affine, Node, NodeId, Rect, Role as NodeRole, Toggled, TreeId, TreeUpdate,
};
use morok_core::{AccessibleActions, AccessibleValue, ElementId, Focusability, NumericValue, Role};
use morok_style::States;
use support::{InputLog, Scene};

const DIALOG: &str = ".dialog { display: flex; flex-direction: column; gap: 10px; padding: 10px; width: 400px; }\n\
    .row { display: flex; gap: 10px; }\n\
    .button { width: 80px; height: 30px; background-color: var(--color-surface); }\n\
    .button:hover { background-color: var(--color-surface-2); }\n\
    .wide { width: 120px; }\n\
    .moved { transform: translateX(100px); }\n\
    .gone { display: none; }";

/// The accessibility tree as an assistive technology holds it: every update is applied to it
/// as an adapter would apply it.
#[derive(Debug, Default)]
struct Mirror {
    nodes: HashMap<NodeId, Node>,
    root: Option<NodeId>,
    focus: Option<NodeId>,
}

impl Mirror {
    /// Applies the update and drops what the tree no longer reaches.
    ///
    /// # Panics
    ///
    /// When the update leaves the tree with a child it holds no node of: an adapter would
    /// refuse such an update.
    fn apply(&mut self, update: &TreeUpdate) {
        assert_eq!(update.tree_id, TreeId::ROOT);
        if let Some(tree) = &update.tree {
            self.root = Some(tree.root);
        }
        for (id, node) in &update.nodes {
            self.nodes.insert(*id, node.clone());
        }
        let root = self.root.expect("the first update names the root");
        let mut reached = Vec::new();
        let mut pending = vec![root];
        while let Some(id) = pending.pop() {
            let node = self
                .nodes
                .get(&id)
                .unwrap_or_else(|| panic!("the tree names {id:?} as a child and holds no node of it"));
            pending.extend(node.children());
            reached.push(id);
        }
        self.nodes.retain(|id, _| reached.contains(id));
        assert!(self.nodes.contains_key(&update.focus), "focus is on a node of the tree");
        self.focus = Some(update.focus);
    }

    fn node(&self, element: ElementId) -> &Node {
        self.nodes
            .get(&NodeId::from(element))
            .expect("the element has a node in the tree")
    }

    fn holds(&self, element: ElementId) -> bool {
        self.nodes.contains_key(&NodeId::from(element))
    }

    fn children(&self, element: ElementId) -> &[NodeId] {
        self.node(element).children()
    }
}

/// A dialog that stacks a title, a checkbox and a row of two buttons; the row has no node.
struct DialogScene {
    scene: Scene,
    mirror: Mirror,
    panel: ElementId,
    title: ElementId,
    row: ElementId,
    save: ElementId,
    cancel: ElementId,
    remember: ElementId,
}

impl DialogScene {
    /// The scene after its first frame, with the whole tree given to the mirror.
    fn new() -> Self {
        let mut scene = Scene::new(DIALOG);
        let panel = scene.add(scene.root(), Role::Element, &["dialog"]);
        let title = scene.add_text(panel, "Save changes?");
        let remember = scene.add(panel, Role::Element, &["button"]);
        let row = scene.add(panel, Role::Element, &["row"]);
        let save = scene.add(row, Role::Element, &["button"]);
        let cancel = scene.add(row, Role::Element, &["button"]);
        let tree = &mut scene.tree;
        tree.set_accessible_role(panel, Some(NodeRole::Dialog));
        tree.set_accessible_name(panel, "Unsaved document");
        tree.set_accessible_role(title, Some(NodeRole::Label));
        for (button, name) in [(save, "Save"), (cancel, "Cancel")] {
            tree.set_accessible_role(button, Some(NodeRole::Button));
            tree.set_accessible_name(button, name);
            tree.set_accessible_actions(button, AccessibleActions::CLICK);
            tree.set_focusability(button, Focusability::TabStop);
        }
        tree.set_accessible_role(remember, Some(NodeRole::CheckBox));
        tree.set_accessible_name(remember, "Remember my choice");
        scene.frame();
        let mut mirror = Mirror::default();
        mirror.apply(&scene.tree.accessibility_tree());
        Self {
            scene,
            mirror,
            panel,
            title,
            row,
            save,
            cancel,
            remember,
        }
    }

    /// Runs a frame and gives its update to the mirror. Returns the nodes the update held.
    fn update(&mut self) -> Vec<NodeId> {
        self.scene.frame();
        let update = self.scene.tree.accessibility_update();
        self.mirror.apply(&update);
        update.nodes.iter().map(|(id, _)| *id).collect()
    }

    fn request(&mut self, action: Action, target: ElementId, data: Option<ActionData>, log: &mut InputLog) {
        let request = ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: NodeId::from(target),
            data,
        };
        self.scene.tree.accessibility_action(&request, log);
    }
}

fn ids(elements: &[ElementId]) -> Vec<NodeId> {
    elements.iter().copied().map(NodeId::from).collect()
}

fn sorted(mut nodes: Vec<NodeId>) -> Vec<NodeId> {
    nodes.sort();
    nodes
}

#[test]
fn every_element_with_a_role_has_a_node_and_the_others_hand_their_children_up() {
    let dialog = DialogScene::new();
    let root = dialog.scene.root();

    assert_eq!(dialog.mirror.root, Some(NodeId::from(root)));
    assert_eq!(dialog.mirror.node(root).role(), NodeRole::Window);
    assert_eq!(dialog.mirror.children(root), ids(&[dialog.panel]));
    assert_eq!(
        dialog.mirror.children(dialog.panel),
        ids(&[dialog.title, dialog.remember, dialog.save, dialog.cancel]),
        "the buttons of the row without a node are children of the dialog, in tree order"
    );
    assert!(!dialog.mirror.holds(dialog.row));
    assert_eq!(dialog.mirror.node(dialog.panel).role(), NodeRole::Dialog);
    assert_eq!(dialog.mirror.node(dialog.save).role(), NodeRole::Button);
    assert_eq!(dialog.mirror.node(dialog.remember).role(), NodeRole::CheckBox);
}

#[test]
fn a_node_carries_the_name_and_the_value_of_its_element() {
    let mut dialog = DialogScene::new();

    assert_eq!(dialog.mirror.node(dialog.panel).label(), Some("Unsaved document"));
    assert_eq!(dialog.mirror.node(dialog.save).label(), Some("Save"));
    assert_eq!(dialog.mirror.node(dialog.save).value(), None);
    assert_eq!(
        dialog.mirror.node(dialog.title).value(),
        Some("Save changes?"),
        "a text element gives its text"
    );
    assert_eq!(dialog.mirror.node(dialog.title).label(), None);

    let tree = &mut dialog.scene.tree;
    tree.set_accessible_value(dialog.title, AccessibleValue::Text("Save your changes?"));
    tree.set_accessible_value(dialog.save, AccessibleValue::Text("ready"));
    let volume = NumericValue {
        value: 40.0,
        min: 0.0,
        max: 100.0,
        step: 5.0,
    };
    tree.set_accessible_value(dialog.remember, AccessibleValue::Number(volume));
    dialog.update();

    assert_eq!(dialog.mirror.node(dialog.title).value(), Some("Save your changes?"));
    assert_eq!(dialog.mirror.node(dialog.save).value(), Some("ready"));
    let slider = dialog.mirror.node(dialog.remember);
    assert_eq!(
        (
            slider.numeric_value(),
            slider.min_numeric_value(),
            slider.max_numeric_value(),
            slider.numeric_value_step()
        ),
        (Some(40.0), Some(0.0), Some(100.0), Some(5.0))
    );

    dialog
        .scene
        .tree
        .set_accessible_value(dialog.save, AccessibleValue::None);
    dialog.update();
    assert_eq!(dialog.mirror.node(dialog.save).value(), None);
}

#[test]
fn the_bounds_of_a_node_are_the_box_of_its_element_on_the_surface() {
    let mut dialog = DialogScene::new();
    let save_box = dialog.scene.border_box(dialog.save);

    assert_eq!(
        dialog.mirror.node(dialog.save).bounds(),
        Some(Rect {
            x0: f64::from(save_box.x),
            y0: f64::from(save_box.y),
            x1: f64::from(save_box.right()),
            y1: f64::from(save_box.bottom()),
        })
    );
    assert_eq!(dialog.mirror.node(dialog.save).transform(), None);

    dialog.scene.add_classes(dialog.row, &["moved"]);
    dialog.update();
    let moved = dialog.mirror.node(dialog.save).bounds().expect("a node has bounds");
    assert_eq!(
        (moved.x0, moved.y0),
        (f64::from(save_box.x) + 100.0, f64::from(save_box.y)),
        "a transform of an element around it moves the node with it"
    );
}

#[test]
fn the_root_takes_the_boxes_from_css_pixels_to_device_pixels() {
    let mut dialog = DialogScene::new();
    let root = dialog.scene.root();
    assert_eq!(dialog.mirror.node(root).transform(), Some(&Affine::scale(1.0)));

    dialog.scene.scale = 2.0;
    let changed = dialog.update();

    assert_eq!(dialog.mirror.node(root).transform(), Some(&Affine::scale(2.0)));
    assert_eq!(changed, ids(&[root]), "the boxes themselves stay in CSS pixels");
}

#[test]
fn a_node_carries_the_states_of_its_element() {
    let mut dialog = DialogScene::new();
    assert!(!dialog.mirror.node(dialog.save).is_disabled());
    assert_eq!(dialog.mirror.node(dialog.remember).toggled(), Some(Toggled::False));
    assert_eq!(
        dialog.mirror.node(dialog.save).toggled(),
        None,
        "a button is not checked or unchecked"
    );

    dialog.scene.tree.set_state(dialog.save, States::DISABLED, true);
    dialog.scene.tree.set_state(dialog.remember, States::CHECKED, true);
    let changed = dialog.update();

    assert!(dialog.mirror.node(dialog.save).is_disabled());
    assert_eq!(dialog.mirror.node(dialog.remember).toggled(), Some(Toggled::True));
    assert_eq!(sorted(changed), sorted(ids(&[dialog.save, dialog.remember])));
}

#[test]
fn a_button_made_a_toggle_tells_whether_it_is_checked() {
    let mut dialog = DialogScene::new();

    dialog.scene.tree.set_accessible_toggle(dialog.save, true);
    let changed = dialog.update();
    assert_eq!(dialog.mirror.node(dialog.save).toggled(), Some(Toggled::False));
    assert_eq!(changed, ids(&[dialog.save]));

    dialog.scene.tree.set_state(dialog.save, States::CHECKED, true);
    dialog.update();
    assert_eq!(dialog.mirror.node(dialog.save).toggled(), Some(Toggled::True));

    dialog.scene.tree.set_accessible_toggle(dialog.save, false);
    dialog.update();
    assert_eq!(dialog.mirror.node(dialog.save).toggled(), None);
}

#[test]
fn a_node_offers_the_actions_of_its_element_and_focus_when_the_element_takes_it() {
    let mut dialog = DialogScene::new();
    let save = dialog.mirror.node(dialog.save);
    assert!(save.supports_action(Action::Click) && save.supports_action(Action::Focus));
    assert!(!save.supports_action(Action::Increment));
    let remember = dialog.mirror.node(dialog.remember);
    assert!(!remember.supports_action(Action::Focus) && !remember.supports_action(Action::Click));

    let stepping = AccessibleActions::INCREMENT
        .union(AccessibleActions::DECREMENT)
        .union(AccessibleActions::SET_VALUE);
    dialog.scene.tree.set_accessible_actions(dialog.remember, stepping);
    dialog
        .scene
        .tree
        .set_focusability(dialog.remember, Focusability::Focusable);
    dialog.scene.tree.set_state(dialog.save, States::DISABLED, true);
    dialog.update();

    let remember = dialog.mirror.node(dialog.remember);
    for offered in [Action::Increment, Action::Decrement, Action::SetValue, Action::Focus] {
        assert!(remember.supports_action(offered), "{offered:?} is offered");
    }
    assert!(
        !dialog.mirror.node(dialog.save).supports_action(Action::Focus),
        "a disabled element takes no focus"
    );
}

#[test]
fn an_update_after_a_frame_that_changed_nothing_is_empty() {
    let mut dialog = DialogScene::new();

    assert_eq!(dialog.update(), []);

    let mut log = InputLog::default();
    let save_box = dialog.scene.border_box(dialog.save);
    dialog.scene.move_pointer(save_box.x + 5.0, save_box.y + 5.0, &mut log);
    assert_eq!(
        dialog.update(),
        [],
        "hover repaints the button and changes nothing of its node"
    );
}

#[test]
fn an_update_carries_only_the_nodes_that_changed() {
    let mut dialog = DialogScene::new();

    dialog.scene.tree.set_accessible_name(dialog.cancel, "Discard");
    assert_eq!(dialog.update(), ids(&[dialog.cancel]));
    assert_eq!(dialog.mirror.node(dialog.cancel).label(), Some("Discard"));

    dialog.scene.tree.set_accessible_name(dialog.cancel, "Discard");
    assert_eq!(dialog.update(), [], "the same name again changes nothing");

    dialog.scene.tree.set_text(dialog.title, "Keep the changes?");
    assert_eq!(dialog.update(), ids(&[dialog.title]));
    assert_eq!(dialog.mirror.node(dialog.title).value(), Some("Keep the changes?"));

    dialog
        .scene
        .tree
        .set_accessible_role(dialog.cancel, Some(NodeRole::Link));
    assert_eq!(dialog.update(), ids(&[dialog.cancel]));
    assert_eq!(dialog.mirror.node(dialog.cancel).role(), NodeRole::Link);
}

#[test]
fn a_box_that_moves_changes_the_nodes_it_moves() {
    let mut dialog = DialogScene::new();

    dialog.scene.add_classes(dialog.save, &["wide"]);
    let changed = dialog.update();

    assert_eq!(
        sorted(changed),
        sorted(ids(&[dialog.save, dialog.cancel])),
        "the button grew and pushed the one after it; the rest of the dialog stayed"
    );
}

#[test]
fn an_element_that_joins_the_tree_brings_its_nodes_and_one_that_leaves_takes_them() {
    let mut dialog = DialogScene::new();
    let group = dialog.scene.tree.create(Role::Element);
    let help = dialog.scene.tree.create(Role::Element);
    dialog.scene.tree.set_accessible_role(group, Some(NodeRole::Group));
    dialog.scene.tree.set_accessible_role(help, Some(NodeRole::Button));
    dialog.scene.tree.set_accessible_name(help, "Help");
    dialog.scene.tree.append_child(group, help);

    dialog.scene.tree.append_child(dialog.row, group);
    let changed = dialog.update();
    assert_eq!(
        sorted(changed),
        sorted(ids(&[dialog.panel, group, help])),
        "the dialog got a child, and the child came with what is inside it"
    );
    assert_eq!(dialog.mirror.children(group), ids(&[help]));

    dialog.scene.tree.detach(group);
    assert_eq!(dialog.update(), ids(&[dialog.panel]));
    assert!(!dialog.mirror.holds(group) && !dialog.mirror.holds(help));

    dialog.scene.tree.append_child(dialog.panel, group);
    let changed = dialog.update();
    assert_eq!(
        sorted(changed),
        sorted(ids(&[dialog.panel, group, help])),
        "nodes that come back are new to the tree"
    );

    dialog.scene.tree.remove(group);
    assert_eq!(dialog.update(), ids(&[dialog.panel]));
}

#[test]
fn an_element_that_is_not_rendered_has_no_node() {
    let mut dialog = DialogScene::new();

    dialog.scene.add_classes(dialog.row, &["gone"]);
    assert_eq!(dialog.update(), ids(&[dialog.panel]));
    assert_eq!(
        dialog.mirror.children(dialog.panel),
        ids(&[dialog.title, dialog.remember])
    );

    let gone = dialog.scene.class("gone");
    dialog.scene.tree.remove_class(dialog.row, gone);
    let changed = dialog.update();
    assert_eq!(
        sorted(changed),
        sorted(ids(&[dialog.panel, dialog.save, dialog.cancel]))
    );
}

#[test]
fn an_element_that_loses_its_role_hands_its_children_up() {
    let mut dialog = DialogScene::new();
    let root = dialog.scene.root();

    dialog.scene.tree.set_accessible_role(dialog.panel, None);
    let changed = dialog.update();

    assert_eq!(changed, ids(&[root]));
    assert_eq!(
        dialog.mirror.children(root),
        ids(&[dialog.title, dialog.remember, dialog.save, dialog.cancel])
    );
    assert_eq!(dialog.scene.tree.accessible_role(dialog.panel), None);
}

#[test]
fn focus_is_on_the_node_of_the_focused_element_or_of_the_nearest_one_around_it() {
    let mut dialog = DialogScene::new();
    let root = dialog.scene.root();
    assert_eq!(
        dialog.mirror.focus,
        Some(NodeId::from(root)),
        "with no focus the root has it"
    );

    assert!(dialog.scene.tree.focus(dialog.cancel));
    assert_eq!(dialog.update(), [], "focus changes no node");
    assert_eq!(dialog.mirror.focus, Some(NodeId::from(dialog.cancel)));

    dialog.scene.tree.set_focusability(dialog.row, Focusability::Focusable);
    assert!(dialog.scene.tree.focus(dialog.row));
    dialog.update();
    assert_eq!(
        dialog.mirror.focus,
        Some(NodeId::from(dialog.panel)),
        "the row has no node: the dialog around it stands for it"
    );
}

#[test]
fn a_whole_tree_holds_every_node_again() {
    let mut dialog = DialogScene::new();
    dialog.update();

    let whole = dialog.scene.tree.accessibility_tree();

    assert_eq!(whole.nodes.len(), 6);
    assert!(whole.tree.is_some());
    assert_eq!(dialog.scene.tree.accessibility_update().tree, None);
}

#[test]
fn a_focus_request_moves_focus_as_the_keyboard_would() {
    let mut dialog = DialogScene::new();
    let mut log = InputLog::default();

    dialog.request(Action::Focus, dialog.cancel, None, &mut log);

    assert_eq!(dialog.scene.tree.focused(), Some(dialog.cancel));
    assert!(dialog.scene.tree.states(dialog.cancel).contains(States::FOCUS_VISIBLE));
    assert_eq!(log.accessibility_actions, []);

    dialog.request(Action::Focus, dialog.remember, None, &mut log);
    assert_eq!(
        dialog.scene.tree.focused(),
        Some(dialog.cancel),
        "an element that takes no focus is given none"
    );
}

#[test]
fn an_action_an_element_offers_is_told_to_the_element() {
    let mut dialog = DialogScene::new();
    let mut log = InputLog::default();
    let stepping = AccessibleActions::INCREMENT
        .union(AccessibleActions::DECREMENT)
        .union(AccessibleActions::SET_VALUE);
    dialog.scene.tree.set_accessible_actions(dialog.remember, stepping);

    dialog.request(Action::Click, dialog.save, None, &mut log);
    dialog.request(Action::Increment, dialog.remember, None, &mut log);
    dialog.request(Action::Decrement, dialog.remember, None, &mut log);
    let text = Some(ActionData::Value("yes".into()));
    dialog.request(Action::SetValue, dialog.remember, text, &mut log);
    let number = Some(ActionData::NumericValue(0.5));
    dialog.request(Action::SetValue, dialog.remember, number, &mut log);

    let told: Vec<(ElementId, &str)> = log
        .accessibility_actions
        .iter()
        .map(|(element, action)| (*element, action.as_str()))
        .collect();
    assert_eq!(
        told,
        [
            (dialog.save, "Click"),
            (dialog.remember, "Increment"),
            (dialog.remember, "Decrement"),
            (dialog.remember, "SetText(\"yes\")"),
            (dialog.remember, "SetNumber(0.5)"),
        ]
    );
}

#[test]
fn a_request_an_element_does_not_offer_does_nothing() {
    let mut dialog = DialogScene::new();
    let mut log = InputLog::default();

    dialog.request(Action::Increment, dialog.save, None, &mut log);
    dialog.request(Action::Click, dialog.remember, None, &mut log);
    dialog.request(Action::Click, dialog.row, None, &mut log);
    dialog.request(Action::ScrollIntoView, dialog.save, None, &mut log);

    dialog.scene.tree.set_state(dialog.save, States::DISABLED, true);
    dialog.request(Action::Click, dialog.save, None, &mut log);
    dialog.request(Action::Focus, dialog.save, None, &mut log);

    dialog.scene.tree.remove(dialog.cancel);
    dialog.request(Action::Click, dialog.cancel, None, &mut log);

    assert_eq!(log.accessibility_actions, []);
    assert_eq!(dialog.scene.tree.focused(), None);
}
