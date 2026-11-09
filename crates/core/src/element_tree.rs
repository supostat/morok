use std::mem;

use morok_style::{Class, Declaration, States, StyleSheet, Theme};

use crate::dirty::Dirty;
use crate::element::Element;
use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::frame_scheduler::FrameScheduler;
use crate::geometry::rect::Rect;
use crate::geometry::size::Size;
use crate::geometry::transform::Transform;
use crate::layout::geometry_pass::GeometryPass;
use crate::layout::layout_pass::LayoutPass;
use crate::paint::box_painter::BoxPainter;
use crate::paint::damage::Damage;
use crate::paint::display_list::DisplayList;
use crate::paint::group_transform::local_transform;
use crate::paint::paint_order::PaintOrder;
use crate::paint::paint_pass::{OpenGroup, PaintPass};
use crate::paint::paint_step::PaintStep;
use crate::paint::unpaint::unpaint_subtree;
use crate::role::Role;
use crate::style::computed_style::ComputedStyle;
use crate::style::selector_change::SelectorChange;
use crate::style::style_pass::StylePass;
use crate::style::style_resolver::StyleResolver;
use crate::text::line_box::LineBox;
use crate::text::text_layout_pass::TextLayoutPass;
use crate::text::text_shaper::TextShaper;

/// The retained tree of one surface: its elements, their styles, their boxes and its display list.
///
/// Changing the tree only marks what the change touches. The work is done by the three stages
/// of a frame, called in this order: [`resolve_styles`](Self::resolve_styles),
/// [`compute_layout`](Self::compute_layout), [`build_display_list`](Self::build_display_list).
/// A tree with a [`FrameScheduler`] asks it for a frame whenever a change waits for one.
#[derive(Debug)]
pub struct ElementTree {
    elements: ElementArena,
    root: ElementId,
    style_sheet: StyleSheet,
    theme: Theme,
    /// Device pixels to a CSS pixel on the surface the tree was last laid out for.
    scale: f32,
    matched_rules: Vec<u32>,
    pending: Vec<ElementId>,
    paint_steps: Vec<PaintStep>,
    /// Scratch space for the groups open while the display list is written.
    open_groups: Vec<OpenGroup>,
    paint_order_is_stale: bool,
    /// Whether an element waits for the display list stage.
    has_unpainted_changes: bool,
    display_list: DisplayList,
    previous_display_list: DisplayList,
    damage: Damage,
    /// The damage of changes made since the last display list; it joins the damage of the next one.
    pending_damage: Damage,
    /// Scratch space for texts whose white space collapses.
    processed_text: String,
    frame_scheduler: Option<Box<dyn FrameScheduler>>,
    /// Whether a change waits for a frame that has not begun.
    frame_is_pending: bool,
}

impl ElementTree {
    /// A tree of one element, the root, styled by `style_sheet` under `theme`.
    pub fn new(style_sheet: StyleSheet, theme: Theme) -> Self {
        let mut elements = ElementArena::default();
        let mut root_element = Element::new(Role::Element);
        root_element.is_attached = true;
        let root = elements.insert(root_element);
        elements.invalidate_layout(root);
        Self {
            elements,
            root,
            style_sheet,
            theme,
            scale: 1.0,
            matched_rules: Vec::new(),
            pending: Vec::new(),
            paint_steps: Vec::new(),
            open_groups: Vec::new(),
            paint_order_is_stale: true,
            has_unpainted_changes: true,
            display_list: DisplayList::default(),
            previous_display_list: DisplayList::default(),
            damage: Damage::default(),
            pending_damage: Damage::default(),
            processed_text: String::new(),
            frame_scheduler: None,
            frame_is_pending: true,
        }
    }

    /// Names the host that runs the frames of the tree. A tree that already waits for a frame
    /// asks for it at once.
    pub fn set_frame_scheduler(&mut self, frame_scheduler: Box<dyn FrameScheduler>) {
        if self.frame_is_pending {
            frame_scheduler.request_frame();
        }
        self.frame_scheduler = Some(frame_scheduler);
    }

    pub fn root(&self) -> ElementId {
        self.root
    }

    /// Whether the ID names an element that has not been removed.
    pub fn contains(&self, element: ElementId) -> bool {
        self.elements.contains(element)
    }

    /// Creates an element outside the tree; it takes part in frames once it has a parent.
    pub fn create(&mut self, role: Role) -> ElementId {
        self.elements.insert(Element::new(role))
    }

    /// Adds `child` after the last child of `parent`.
    ///
    /// # Panics
    ///
    /// As [`insert_child`](Self::insert_child) does.
    pub fn append_child(&mut self, parent: ElementId, child: ElementId) {
        let position = self.elements[parent].children.len();
        self.insert_child(parent, position, child);
    }

    /// Adds `child` to `parent` before the child at `position`.
    ///
    /// # Panics
    ///
    /// When `parent` cannot hold children by its role, when `child` already has a parent or is
    /// the root, or when `child` is `parent` or one of its ancestors.
    pub fn insert_child(&mut self, parent: ElementId, position: usize, child: ElementId) {
        assert!(
            self.elements[parent].role.is_container(),
            "{:?} holds no children",
            self.elements[parent].role
        );
        assert!(
            child != self.root && self.elements[child].parent.is_none(),
            "the child is detached before it is inserted"
        );
        assert!(
            !self.is_inclusive_ancestor(child, parent),
            "an element cannot contain itself"
        );
        self.elements[parent].children.insert(position, child);
        self.elements[child].parent = Some(parent);
        if self.elements[parent].is_attached {
            self.attach_subtree(child);
            self.elements.invalidate_layout(parent);
            self.paint_order_is_stale = true;
            self.has_unpainted_changes = true;
            self.request_frame();
        }
    }

    /// Takes the element out of its parent, keeping it and its descendants for a later insertion.
    ///
    /// # Panics
    ///
    /// When the element has no parent.
    pub fn detach(&mut self, element: ElementId) {
        let parent = self.elements[element]
            .parent
            .take()
            .expect("the element has a parent to be detached from");
        self.elements[parent].children.retain(|child| *child != element);
        if self.elements[parent].is_attached {
            self.detach_subtree(element);
            self.elements.invalidate_layout(parent);
            self.paint_order_is_stale = true;
            self.has_unpainted_changes = true;
            self.request_frame();
        }
    }

    /// Removes the element and its descendants; their IDs name nothing from now on.
    ///
    /// # Panics
    ///
    /// When the element is the root.
    pub fn remove(&mut self, element: ElementId) {
        assert!(element != self.root, "the root cannot be removed");
        if self.elements[element].parent.is_some() {
            self.detach(element);
        }
        self.pending.clear();
        self.pending.push(element);
        while let Some(removed) = self.pending.pop() {
            let removed = self.elements.remove(removed);
            self.pending.extend(removed.children);
        }
    }

    pub fn role(&self, element: ElementId) -> Role {
        self.elements[element].role
    }

    pub fn parent(&self, element: ElementId) -> Option<ElementId> {
        self.elements[element].parent
    }

    pub fn children(&self, element: ElementId) -> &[ElementId] {
        &self.elements[element].children
    }

    pub fn classes(&self, element: ElementId) -> &[Class] {
        &self.elements[element].classes
    }

    pub fn add_class(&mut self, element: ElementId, class: Class) {
        let classes = &mut self.elements[element].classes;
        if classes.contains(&class) {
            return;
        }
        classes.push(class);
        self.mark_selector_change(element, SelectorChange::ClassAdded);
    }

    pub fn remove_class(&mut self, element: ElementId, class: Class) {
        let classes = &mut self.elements[element].classes;
        let Some(position) = classes.iter().position(|held| *held == class) else {
            return;
        };
        let last = classes.len() - 1;
        classes.swap(position, last);
        self.mark_selector_change(element, SelectorChange::ClassLeaving);
        self.elements[element].classes.pop();
    }

    pub fn states(&self, element: ElementId) -> States {
        self.elements[element].states
    }

    /// Turns the given states of the element on or off; its other states stay as they are.
    pub fn set_state(&mut self, element: ElementId, state: States, is_on: bool) {
        let before = self.elements[element].states;
        let after = if is_on {
            before.union(state)
        } else {
            before.difference(state)
        };
        if after == before {
            return;
        }
        self.elements[element].states = after;
        self.mark_selector_change(element, SelectorChange::States { before });
    }

    /// Sets one property on the element directly. The declaration overrides every rule of the
    /// style sheet and replaces the inline declaration of the same property.
    pub fn set_inline(&mut self, element: ElementId, declaration: Declaration) {
        self.elements[element].inline_style.set(declaration);
        self.mark_style(element);
    }

    /// Drops every inline declaration of the element.
    pub fn clear_inline(&mut self, element: ElementId) {
        self.elements[element].inline_style.clear();
        self.mark_style(element);
    }

    pub fn text(&self, element: ElementId) -> &str {
        &self.elements[element].text
    }

    /// # Panics
    ///
    /// When the element is not a [`Role::Text`].
    pub fn set_text(&mut self, element: ElementId, text: &str) {
        let held = &mut self.elements[element];
        assert!(held.role == Role::Text, "{:?} holds no text", held.role);
        if held.text == text {
            return;
        }
        held.text.clear();
        held.text.push_str(text);
        held.dirty.insert(Dirty::PAINT);
        let is_attached = held.is_attached;
        self.elements.invalidate_text(element);
        if is_attached {
            self.elements.invalidate_layout(element);
            self.has_unpainted_changes = true;
            self.request_frame();
        }
    }

    /// Switches the active theme: every element is styled, laid out and painted again against
    /// the new token values.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.has_unpainted_changes = true;
        self.pending.clear();
        self.pending.push(self.root);
        while let Some(id) = self.pending.pop() {
            let element = &mut self.elements[id];
            element
                .dirty
                .insert(Dirty::STYLE.union(Dirty::STYLE_BELOW).union(Dirty::PAINT));
            self.pending.extend(&element.children);
        }
        self.forget_layout();
        self.request_frame();
    }

    /// The style of the element as the last style stage resolved it.
    pub fn style(&self, element: ElementId) -> &ComputedStyle {
        &self.elements[element].style
    }

    /// The border box of the element on the surface, as the last layout stage placed it.
    pub fn border_box(&self, element: ElementId) -> Rect {
        self.elements[element].border_box
    }

    /// The rectangle around the border box as the surface shows it: moved by the transforms
    /// of the element and of its ancestors, as a browser's bounding client rect is.
    ///
    /// # Panics
    ///
    /// When a change waits for the style stage.
    pub fn surface_box(&self, element: ElementId) -> Rect {
        assert!(
            !self.elements[self.root]
                .dirty
                .intersects(Dirty::STYLE.union(Dirty::STYLE_BELOW)),
            "styles are resolved before boxes are read"
        );
        let mut on_surface = Transform::IDENTITY;
        let mut current = Some(element);
        while let Some(id) = current {
            let transformed = &self.elements[id];
            on_surface = on_surface.then(&local_transform(transformed, &self.style_sheet, &self.theme));
            current = transformed.parent;
        }
        on_surface.bounds_of(&self.elements[element].border_box)
    }

    /// The line boxes of a text element as the last layout stage placed them, top to bottom.
    /// Another element has none.
    pub fn line_boxes(&self, element: ElementId) -> &[LineBox] {
        &self.elements[element].text_layout.lines
    }

    pub fn display_list(&self) -> &DisplayList {
        &self.display_list
    }

    /// What changed on the surface between the last two display lists.
    pub fn damage(&self) -> &Damage {
        &self.damage
    }

    /// The style stage: resolves the style of every element a change marked, and of the
    /// elements that inherit from them. Returns the number of elements resolved.
    pub fn resolve_styles(&mut self) -> usize {
        self.frame_is_pending = false;
        let outcome = StylePass {
            elements: &mut self.elements,
            resolver: StyleResolver {
                style_sheet: &self.style_sheet,
                theme: &self.theme,
            },
            matched_rules: &mut self.matched_rules,
            pending: &mut self.pending,
        }
        .run(self.root);
        self.paint_order_is_stale |= outcome.reorders_paint;
        self.has_unpainted_changes |= outcome.repaints || outcome.reorders_paint;
        outcome.resolved_elements
    }

    /// The layout stage: lays the tree out in a surface of the size of `viewport` that shows
    /// a CSS pixel with `scale` device pixels. Elements whose boxes cannot have changed keep
    /// them. Returns the number of elements laid out.
    ///
    /// The scale is a part of layout: a line of text is as high as its font is in whole device
    /// pixels, so a surface that moves to a display of another scale lays its texts out again.
    ///
    /// # Panics
    ///
    /// When a change waits for the style stage.
    pub fn compute_layout(&mut self, viewport: Size, scale: f32, text_shaper: &mut dyn TextShaper) -> usize {
        assert!(
            !self.elements[self.root]
                .dirty
                .intersects(Dirty::STYLE.union(Dirty::STYLE_BELOW)),
            "styles are resolved before layout"
        );
        if scale.to_bits() != self.scale.to_bits() {
            self.scale = scale;
            self.forget_layout();
        }
        let laid_out_elements = LayoutPass {
            elements: &mut self.elements,
            style_sheet: &self.style_sheet,
            theme: &self.theme,
            text_shaper: &mut *text_shaper,
            scale,
            processed_text: &mut self.processed_text,
            laid_out_elements: 0,
        }
        .run(self.root, viewport);
        self.elements.layout_is_computed();
        self.has_unpainted_changes |= GeometryPass {
            elements: &mut self.elements,
            repaints: false,
        }
        .run(self.root);
        self.has_unpainted_changes |= TextLayoutPass {
            elements: &mut self.elements,
            theme: &self.theme,
            text_shaper,
            scale,
            processed_text: &mut self.processed_text,
        }
        .run();
        laid_out_elements
    }

    /// The display list stage: rebuilds the display items of the elements that changed and
    /// collects the damage. Returns the number of elements whose items were rebuilt.
    ///
    /// # Panics
    ///
    /// When a change waits for the layout stage.
    pub fn build_display_list(&mut self) -> usize {
        assert!(
            !self.elements.layout_is_stale(),
            "layout is computed before the display list"
        );
        mem::swap(&mut self.damage, &mut self.pending_damage);
        self.pending_damage.clear();
        if !self.has_unpainted_changes {
            return 0;
        }
        if self.paint_order_is_stale {
            PaintOrder {
                elements: &mut self.elements,
                steps: &mut self.paint_steps,
                damage: &mut self.damage,
                clipping_ancestors: &mut self.pending,
            }
            .rebuild(self.root);
            self.paint_order_is_stale = false;
        }
        mem::swap(&mut self.display_list, &mut self.previous_display_list);
        let rebuilt_elements = PaintPass {
            elements: &mut self.elements,
            painter: BoxPainter {
                style_sheet: &self.style_sheet,
                theme: &self.theme,
            },
            style_sheet: &self.style_sheet,
            theme: &self.theme,
            steps: &self.paint_steps,
            previous: &self.previous_display_list,
            display_list: &mut self.display_list,
            damage: &mut self.damage,
            open_groups: &mut self.open_groups,
        }
        .run();
        self.has_unpainted_changes = false;
        rebuilt_elements
    }

    /// Drops the cached boxes and the lines of every element: all of them are laid out again.
    fn forget_layout(&mut self) {
        self.pending.clear();
        self.pending.push(self.root);
        while let Some(id) = self.pending.pop() {
            let element = &mut self.elements[id];
            element.layout_cache.clear();
            self.pending.extend(&element.children);
            self.elements.invalidate_text(id);
        }
        self.elements.invalidate_layout(self.root);
    }

    fn is_inclusive_ancestor(&self, ancestor: ElementId, of: ElementId) -> bool {
        let mut current = Some(of);
        while let Some(id) = current {
            if id == ancestor {
                return true;
            }
            current = self.elements[id].parent;
        }
        false
    }

    /// Brings a subtree into the frames of the tree: every element in it is styled against its
    /// new surroundings.
    fn attach_subtree(&mut self, subtree: ElementId) {
        self.pending.clear();
        self.pending.push(subtree);
        while let Some(id) = self.pending.pop() {
            let element = &mut self.elements[id];
            element.is_attached = true;
            element
                .dirty
                .insert(Dirty::STYLE.union(Dirty::STYLE_BELOW).union(Dirty::PAINT));
            self.pending.extend(&element.children);
        }
        self.elements.mark(subtree, Dirty::STYLE, Dirty::STYLE_BELOW);
    }

    fn detach_subtree(&mut self, subtree: ElementId) {
        unpaint_subtree(&mut self.elements, &mut self.pending_damage, subtree);
        self.pending.clear();
        self.pending.push(subtree);
        while let Some(id) = self.pending.pop() {
            let element = &mut self.elements[id];
            element.is_attached = false;
            element.containing_block = None;
            self.pending.extend(&element.children);
        }
    }

    fn mark_style(&mut self, element: ElementId) {
        if self.elements[element].is_attached {
            self.elements.mark(element, Dirty::STYLE, Dirty::STYLE_BELOW);
            self.request_frame();
        }
    }

    /// Asks the host for a frame unless a frame that has not begun already waits. A change made
    /// once the first stage ran is too late for the running frame and asks for the next one.
    fn request_frame(&mut self) {
        if self.frame_is_pending {
            return;
        }
        self.frame_is_pending = true;
        if let Some(frame_scheduler) = &self.frame_scheduler {
            frame_scheduler.request_frame();
        }
    }

    /// Marks for the style stage the element and those of its children that a child rule
    /// styles differently after the change.
    fn mark_selector_change(&mut self, element: ElementId, change: SelectorChange) {
        if !self.elements[element].is_attached {
            return;
        }
        self.request_frame();
        self.elements.mark(element, Dirty::STYLE, Dirty::STYLE_BELOW);
        let resolver = StyleResolver {
            style_sheet: &self.style_sheet,
            theme: &self.theme,
        };
        for position in 0..self.elements[element].children.len() {
            let parent = &self.elements[element];
            let child = parent.children[position];
            let (before, after) = change.targets(parent);
            let child_target = self.elements[child].selector_target();
            if resolver.child_rules_tell_parents_apart(child_target, before, after) {
                self.elements.mark(child, Dirty::STYLE, Dirty::STYLE_BELOW);
            }
        }
    }
}
