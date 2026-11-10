use std::time::Duration;

use morok_components::view::{Mount, SurfaceId, Surfaces};
use morok_core::{ElementId, ElementTree, Rect, Role, Size, TextShaper};
use morok_reactive::Runtime;
use morok_style::{States, StyleSheet, Theme};

use crate::element_box::ElementBox;
use crate::media_source::{IconSource, ImageSource};
use crate::rendering::DEVICE_SCALE;

const ROOT_NODE: &str = "root";
const TEXT_NODE: &str = "text";

/// The element tree of a case under one theme, on a surface with a signal graph of its own. A
/// case builds it through the methods that add elements and mount components; both sides of
/// the stand then read the same tree.
#[derive(Debug)]
pub struct Scene {
    runtime: Runtime<Surfaces>,
    surfaces: Surfaces,
    surface: SurfaceId,
    style_sheet: StyleSheet,
    /// The icons of the scene with the elements that show them.
    icons: Vec<(ElementId, IconSource)>,
    /// The images of the scene with the elements that show them.
    images: Vec<(ElementId, ImageSource)>,
}

impl Scene {
    pub(crate) fn new(style_sheet: StyleSheet, theme: Theme) -> Self {
        let mut runtime = Runtime::new();
        let mut surfaces = Surfaces::new();
        let surface = surfaces.open(&mut runtime, style_sheet.clone(), theme);
        Self {
            runtime,
            surfaces,
            surface,
            style_sheet,
            icons: Vec::new(),
            images: Vec::new(),
        }
    }

    pub fn root(&self) -> ElementId {
        self.tree().root()
    }

    /// Mounts a component of the kit as the last child of `parent` and returns the element
    /// it built.
    pub fn mount(&mut self, parent: ElementId, component: impl FnOnce(&mut Mount<'_>) -> ElementId) -> ElementId {
        self.surfaces.mount(&mut self.runtime, self.surface, parent, component)
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
        self.tree_mut().set_text(element, text);
        element
    }

    /// Adds an icon with the given classes as the last child of `parent`; both sides draw the
    /// same SVG.
    ///
    /// # Panics
    ///
    /// When the stylesheets of the suite name no such class.
    pub fn add_icon(&mut self, parent: ElementId, class_names: &[&str], icon: IconSource) -> ElementId {
        let element = self.add_with_role(parent, Role::Icon, class_names);
        self.icons.push((element, icon));
        element
    }

    /// Adds an image with the given classes as the last child of `parent`; both sides show the
    /// same encoded picture.
    ///
    /// # Panics
    ///
    /// When the stylesheets of the suite name no such class.
    pub fn add_image(&mut self, parent: ElementId, class_names: &[&str], image: ImageSource) -> ElementId {
        let element = self.add_with_role(parent, Role::Image, class_names);
        self.images.push((element, image));
        element
    }

    /// Puts the element in the given states.
    pub fn set_state(&mut self, element: ElementId, states: States) {
        self.tree_mut().set_state(element, states, true);
    }

    /// The elements that have the class, in document order.
    pub fn elements_with_class(&self, class_name: &str) -> Vec<ElementId> {
        let class = self
            .style_sheet
            .class_named(class_name)
            .unwrap_or_else(|| panic!("the stylesheets of the suite name no class `{class_name}`"));
        self.elements()
            .into_iter()
            .filter(|element| self.tree().classes(*element).contains(&class))
            .collect()
    }

    pub(crate) fn icons(&self) -> &[(ElementId, IconSource)] {
        &self.icons
    }

    pub(crate) fn images(&self) -> &[(ElementId, ImageSource)] {
        &self.images
    }

    pub(crate) fn tree_mut(&mut self) -> &mut ElementTree {
        self.surfaces.surface_mut(self.surface).tree_mut()
    }

    /// The icon of the element, when it shows one.
    pub(crate) fn icon_of(&self, element: ElementId) -> Option<IconSource> {
        self.icons
            .iter()
            .find(|(shown_by, _)| *shown_by == element)
            .map(|(_, icon)| *icon)
    }

    /// The image of the element, when it shows one.
    pub(crate) fn image_of(&self, element: ElementId) -> Option<ImageSource> {
        self.images
            .iter()
            .find(|(shown_by, _)| *shown_by == element)
            .map(|(_, image)| *image)
    }

    pub(crate) fn tree(&self) -> &ElementTree {
        self.surfaces.surface(self.surface).tree()
    }

    /// The names of the classes of the element, in the order it was given them.
    pub(crate) fn class_names(&self, element: ElementId) -> impl Iterator<Item = &str> {
        self.tree()
            .classes(element)
            .iter()
            .map(|class| self.style_sheet.class_style(*class).name.as_ref())
    }

    /// Runs the effects that wait: what the components bound to signals reaches the tree.
    ///
    /// # Panics
    ///
    /// When an effect of a component keeps waking itself.
    pub(crate) fn flush(&mut self) {
        if let Err(cycle) = self.runtime.flush(&mut self.surfaces) {
            panic!("{cycle}");
        }
    }

    /// Runs the frame at `now`, counted from the first frame, on a surface of `viewport` at
    /// the scale of the stand: the effects that wait, then the stages of the tree.
    pub(crate) fn frame(&mut self, viewport: Size, text_shaper: &mut dyn TextShaper, now: Duration) {
        self.flush();
        let tree = self.tree_mut();
        tree.resolve_styles(now);
        tree.compute_layout(viewport, DEVICE_SCALE, text_shaper);
        tree.build_display_list();
    }

    /// The elements in document order: every element before its children, the children in
    /// their order.
    pub(crate) fn elements(&self) -> Vec<ElementId> {
        let mut elements = Vec::new();
        let tree = self.tree();
        let mut unvisited = vec![tree.root()];
        while let Some(element) = unvisited.pop() {
            elements.push(element);
            unvisited.extend(tree.children(element).iter().rev());
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
                let description = match (classes.is_empty(), self.tree().role(element)) {
                    (false, _) => classes.as_str(),
                    (true, Role::Text) => TEXT_NODE,
                    (true, Role::Icon) => "icon",
                    (true, Role::Image) => "image",
                    (true, _) if element == self.root() => ROOT_NODE,
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
        let classes: Vec<_> = class_names
            .iter()
            .map(|name| {
                self.style_sheet
                    .class_named(name)
                    .unwrap_or_else(|| panic!("the stylesheets of the suite name no class `{name}`"))
            })
            .collect();
        let tree = self.tree_mut();
        let element = tree.create(role);
        for class in classes {
            tree.add_class(element, class);
        }
        tree.append_child(parent, element);
        element
    }
}
