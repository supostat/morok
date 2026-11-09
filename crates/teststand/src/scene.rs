use morok_core::{ElementId, ElementTree, Rect, Role, Size, TextShaper};
use morok_style::{States, StyleSheet, Theme};

use crate::element_box::ElementBox;
use crate::rendering::DEVICE_SCALE;

const ROOT_NODE: &str = "root";
const TEXT_NODE: &str = "text";

/// The element tree of a case under one theme. A case builds it through the methods that add
/// elements; both sides of the stand then read the same tree.
#[derive(Debug)]
pub struct Scene {
    tree: ElementTree,
    style_sheet: StyleSheet,
}

impl Scene {
    pub(crate) fn new(style_sheet: StyleSheet, theme: Theme) -> Self {
        Self {
            tree: ElementTree::new(style_sheet.clone(), theme),
            style_sheet,
        }
    }

    pub fn root(&self) -> ElementId {
        self.tree.root()
    }

    /// Adds a box with the given classes as the last child of `parent`.
    ///
    /// # Panics
    ///
    /// When the stylesheets of the suite name no such class.
    pub fn add(&mut self, parent: ElementId, class_names: &[&str]) -> ElementId {
        self.add_with_role(parent, Role::Element, class_names)
    }

    /// Adds a text with the given classes as the last child of `parent`.
    ///
    /// # Panics
    ///
    /// When the stylesheets of the suite name no such class.
    pub fn add_text(&mut self, parent: ElementId, class_names: &[&str], text: &str) -> ElementId {
        let element = self.add_with_role(parent, Role::Text, class_names);
        self.tree.set_text(element, text);
        element
    }

    /// Puts the element in the given states for the whole case.
    pub fn set_state(&mut self, element: ElementId, states: States) {
        self.tree.set_state(element, states, true);
    }

    pub(crate) fn tree(&self) -> &ElementTree {
        &self.tree
    }

    /// The names of the classes of the element, in the order it was given them.
    pub(crate) fn class_names(&self, element: ElementId) -> impl Iterator<Item = &str> {
        self.tree
            .classes(element)
            .iter()
            .map(|class| self.style_sheet.class_style(*class).name.as_ref())
    }

    /// Runs the stages of a frame on a surface of `viewport` at the scale of the stand.
    pub(crate) fn frame(&mut self, viewport: Size, text_shaper: &mut dyn TextShaper) {
        self.tree.resolve_styles();
        self.tree.compute_layout(viewport, DEVICE_SCALE, text_shaper);
        self.tree.build_display_list();
    }

    /// The elements in document order: every element before its children, the children in
    /// their order.
    pub(crate) fn elements(&self) -> Vec<ElementId> {
        let mut elements = Vec::new();
        let mut unvisited = vec![self.tree.root()];
        while let Some(element) = unvisited.pop() {
            elements.push(element);
            unvisited.extend(self.tree.children(element).iter().rev());
        }
        elements
    }

    /// A name for every element in document order: its position, then its classes, or what it
    /// is when it has none.
    pub(crate) fn node_names(&self) -> Vec<String> {
        self.elements()
            .into_iter()
            .enumerate()
            .map(|(position, element)| {
                let mut classes = String::new();
                for name in self.class_names(element) {
                    classes.push('.');
                    classes.push_str(name);
                }
                let description = match (classes.is_empty(), self.tree.role(element)) {
                    (false, _) => classes.as_str(),
                    (true, Role::Text) => TEXT_NODE,
                    (true, _) if element == self.tree.root() => ROOT_NODE,
                    (true, _) => "element",
                };
                format!("#{position} {description}")
            })
            .collect()
    }

    /// The boxes of the elements in document order, as the last frame laid them out; `boxes`
    /// are their rectangles in the same order.
    pub(crate) fn element_boxes(&self, boxes: impl IntoIterator<Item = Rect>) -> Vec<ElementBox> {
        self.node_names()
            .into_iter()
            .zip(boxes)
            .map(|(node, rect)| ElementBox::new(node, rect))
            .collect()
    }

    fn add_with_role(&mut self, parent: ElementId, role: Role, class_names: &[&str]) -> ElementId {
        let element = self.tree.create(role);
        for name in class_names {
            let class = self
                .style_sheet
                .class_named(name)
                .unwrap_or_else(|| panic!("the stylesheets of the suite name no class `{name}`"));
            self.tree.add_class(element, class);
        }
        self.tree.append_child(parent, element);
        element
    }
}
