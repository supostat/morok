use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{ElementId, ElementTree, Key, KeyEvent, KeyPhase, Propagation};

use crate::styles::CLASSES;
use crate::toolbar_part::ToolbarPart;
use crate::toolbar_step::ToolbarStep;
use crate::view::{ElementEvent, Mount, Property};

/// The bar at the top of a window: the controls an application puts there, around the title
/// of the window. The left and right arrow keys move focus from one control to the next and
/// Home and End to the first and the last, unless the focused control uses the key itself, as
/// a text field does.
#[derive(Debug)]
pub struct Toolbar {
    parts: Vec<ToolbarPart>,
    label: Property<String>,
}

impl Toolbar {
    /// A toolbar that holds nothing yet.
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
            label: Property::Fixed(String::new()),
        }
    }

    /// Adds the control `item` builds, after what was added before.
    #[must_use]
    pub fn item(mut self, item: impl FnOnce(&mut Mount<'_>) -> ElementId + 'static) -> Self {
        self.parts.push(ToolbarPart::Item(Box::new(item)));
        self
    }

    /// Adds the title of the window, in bold.
    #[must_use]
    pub fn title(mut self, title: impl Into<Property<String>>) -> Self {
        self.parts.push(ToolbarPart::Title(title.into()));
        self
    }

    /// Adds room that takes what the other parts leave: what is added next stands at the far
    /// end of the bar.
    #[must_use]
    pub fn spacer(mut self) -> Self {
        self.parts.push(ToolbarPart::Spacer);
        self
    }

    /// What an assistive technology calls the toolbar.
    #[must_use]
    pub fn label(mut self, label: impl Into<Property<String>>) -> Self {
        self.label = label.into();
        self
    }

    /// Builds the toolbar: a row of its parts.
    pub fn mount(self, mount: &mut Mount<'_>) -> ElementId {
        let Self { parts, label } = self;
        let mut part_elements = Vec::with_capacity(parts.len());
        let mut items = Vec::new();
        for part in parts {
            part_elements.push(match part {
                ToolbarPart::Item(item) => {
                    let item = item(mount);
                    items.push(item);
                    item
                }
                ToolbarPart::Title(title) => mount.text(title).class(CLASSES.toolbar_title).id(),
                ToolbarPart::Spacer => mount.element().class(CLASSES.toolbar_spacer).id(),
            });
        }
        let asked_step = mount.runtime().signal(None);
        mount.effect(move |runtime, tree| {
            if let Some(step) = runtime.get(asked_step) {
                move_focus(tree, &items, step);
            }
        });
        let mut toolbar = mount.element();
        toolbar
            .class(CLASSES.toolbar)
            .accessible_role(AccessibleRole::Toolbar)
            .accessible_name(label)
            .on_event(move |runtime, event| match event {
                ElementEvent::Key(key) => match step_of(key) {
                    Some(step) => {
                        runtime.set(asked_step, Some(step));
                        Propagation::Stop
                    }
                    None => Propagation::Continue,
                },
                _ => Propagation::Continue,
            });
        for part in part_elements {
            toolbar.child(part);
        }
        toolbar.id()
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

/// Where the key moves focus among the controls; none for a key that moves none.
fn step_of(key: &KeyEvent) -> Option<ToolbarStep> {
    if key.phase != KeyPhase::Pressed || !key.modifiers.is_empty() {
        return None;
    }
    match key.key {
        Key::ArrowRight => Some(ToolbarStep::Next),
        Key::ArrowLeft => Some(ToolbarStep::Previous),
        Key::Home => Some(ToolbarStep::First),
        Key::End => Some(ToolbarStep::Last),
        _ => None,
    }
}

/// Moves focus by the step from the control that holds it to the nearest control that takes
/// focus that way; at the ends of the bar focus stays where it is.
fn move_focus(tree: &mut ElementTree, items: &[ElementId], step: ToolbarStep) {
    let holds_focus = tree
        .focused()
        .and_then(|focused| items.iter().position(|item| is_inside(tree, *item, focused)));
    let candidates: Vec<usize> = match (step, holds_focus) {
        (ToolbarStep::First, _) | (ToolbarStep::Next, None) => (0..items.len()).collect(),
        (ToolbarStep::Last, _) | (ToolbarStep::Previous, None) => (0..items.len()).rev().collect(),
        (ToolbarStep::Next, Some(position)) => (position + 1..items.len()).collect(),
        (ToolbarStep::Previous, Some(position)) => (0..position).rev().collect(),
    };
    for candidate in candidates {
        if focus_inside(tree, items[candidate]) {
            return;
        }
    }
}

/// Gives focus to the element, or to the first element inside it that takes focus. Returns
/// whether one did.
fn focus_inside(tree: &mut ElementTree, element: ElementId) -> bool {
    if tree.focus(element) {
        return true;
    }
    let children = tree.children(element).to_vec();
    children.into_iter().any(|child| focus_inside(tree, child))
}

/// Whether `element` is `subtree` or lies inside it.
fn is_inside(tree: &ElementTree, subtree: ElementId, element: ElementId) -> bool {
    let mut around = Some(element);
    while let Some(current) = around {
        if current == subtree {
            return true;
        }
        around = tree.parent(current);
    }
    false
}
