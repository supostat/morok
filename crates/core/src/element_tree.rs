use std::mem;
use std::time::Duration;

use accesskit::{Action, ActionData, ActionRequest, TreeUpdate};
use morok_style::{Class, Declaration, States, StyleSheet, Theme};

use crate::accessibility::accessibility_action::AccessibilityAction;
use crate::accessibility::accessibility_pass::AccessibilityPass;
use crate::accessibility::accessible_actions::AccessibleActions;
use crate::accessibility::accessible_value::AccessibleValue;
use crate::accessibility::node_identity::element_of;
use crate::accessibility::semantics::Semantics;
use crate::dirty::Dirty;
use crate::element::Element;
use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::frame_requests::FrameRequests;
use crate::frame_scheduler::FrameScheduler;
use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::geometry::size::Size;
use crate::geometry::transform::Transform;
use crate::input::focus_manager::FocusManager;
use crate::input::focusability::Focusability;
use crate::input::hit_test::HitTest;
use crate::input::input_handler::InputHandler;
use crate::input::input_modality::InputModality;
use crate::input::input_state::InputState;
use crate::input::key_chord::KeyChord;
use crate::input::key_event::KeyEvent;
use crate::input::key_router::KeyRouter;
use crate::input::pointer_input::PointerInput;
use crate::input::pointer_input_kind::PointerInputKind;
use crate::input::pointer_router::PointerRouter;
use crate::input::pointer_targets::PointerTargets;
use crate::input::shortcut::Shortcut;
use crate::input::shortcut_id::ShortcutId;
use crate::input::shortcut_scope::ShortcutScope;
use crate::layout::geometry_pass::GeometryPass;
use crate::layout::layout_pass::LayoutPass;
use crate::media::canvas_id::CanvasId;
use crate::media::icon_id::IconId;
use crate::media::image_id::ImageId;
use crate::media::media_content::MediaContent;
use crate::motion::motion_resolver::MotionResolver;
use crate::motion::running_spring::RunningSpring;
use crate::motion::spring::Spring;
use crate::motion::spring_id::SpringId;
use crate::motion::spring_sample::SpringSample;
use crate::motion::timeline::Timeline;
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
use crate::style::selector_marker::SelectorMarker;
use crate::style::style_pass::StylePass;
use crate::style::style_resolver::StyleResolver;
use crate::style::style_scratch::StyleScratch;
use crate::text::line_box::LineBox;
use crate::text::text_layout_pass::TextLayoutPass;
use crate::text::text_shaper::TextShaper;

/// The retained tree of one surface: its elements, their styles, their boxes and its display list.
///
/// Changing the tree only marks what the change touches. The work is done by the three stages
/// of a frame, called in this order: [`resolve_styles`](Self::resolve_styles),
/// [`compute_layout`](Self::compute_layout), [`build_display_list`](Self::build_display_list).
/// A tree with a [`FrameScheduler`] asks it for a frame whenever a change waits for one, and for
/// every next frame while a transition, an animation or a spring runs.
///
/// The tree drives the `hover`, `active` and focus states of its elements from the input it is
/// given, and routes that input to the elements through an [`InputHandler`].
///
/// An element with an accessible role has a node in the accessibility tree; the root always
/// has one. After the stages of a frame [`accessibility_update`](Self::accessibility_update)
/// tells what the frame changed of that tree.
#[derive(Debug)]
pub struct ElementTree {
    elements: ElementArena,
    root: ElementId,
    style_sheet: StyleSheet,
    theme: Theme,
    /// Device pixels to a CSS pixel on the surface the tree was last laid out for.
    scale: f32,
    timeline: Timeline,
    style_scratch: StyleScratch,
    pending: Vec<ElementId>,
    /// The elements whose transitions or animations go on after the last frame.
    moving_elements: Vec<ElementId>,
    springs: Vec<RunningSpring>,
    /// The serial number of the last spring started.
    last_spring: u64,
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
    input: InputState,
    /// The number of the last accessibility update worked out.
    accessibility_updates: u64,
    frame_requests: FrameRequests,
}

impl ElementTree {
    /// A tree of one element, the root, styled by `style_sheet` under `theme`.
    pub fn new(style_sheet: StyleSheet, theme: Theme) -> Self {
        let mut elements = ElementArena::default();
        let mut root_element = Element::new(Role::Element);
        root_element.is_attached = true;
        root_element.semantics = Some(Box::new(Semantics::new(accesskit::Role::Window)));
        let root = elements.insert(root_element);
        elements.invalidate_layout(root);
        Self {
            elements,
            root,
            style_sheet,
            theme,
            scale: 1.0,
            timeline: Timeline::default(),
            style_scratch: StyleScratch::default(),
            pending: Vec::new(),
            moving_elements: Vec::new(),
            springs: Vec::new(),
            last_spring: 0,
            paint_steps: Vec::new(),
            open_groups: Vec::new(),
            paint_order_is_stale: true,
            has_unpainted_changes: true,
            display_list: DisplayList::default(),
            previous_display_list: DisplayList::default(),
            damage: Damage::default(),
            pending_damage: Damage::default(),
            processed_text: String::new(),
            input: InputState::default(),
            accessibility_updates: 0,
            frame_requests: FrameRequests::awaiting_first_frame(),
        }
    }

    /// Names the host that runs the frames of the tree. A tree that already waits for a frame
    /// asks for it at once.
    pub fn set_frame_scheduler(&mut self, frame_scheduler: Box<dyn FrameScheduler>) {
        self.frame_requests.set_scheduler(frame_scheduler);
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
            !self.elements.is_inclusive_ancestor(child, parent),
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
        if self.elements[element].is_attached {
            self.pointer_targets().forget_subtree(element);
            self.focus_manager().forget_subtree(element);
        }
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
        let elements = &self.elements;
        self.input.shortcuts.retain(|shortcut| match shortcut.scope {
            ShortcutScope::Surface => true,
            ShortcutScope::Element(scope) => elements.contains(scope),
        });
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
        self.selector_marker().mark(element, SelectorChange::ClassAdded);
    }

    pub fn remove_class(&mut self, element: ElementId, class: Class) {
        let classes = &mut self.elements[element].classes;
        let Some(position) = classes.iter().position(|held| *held == class) else {
            return;
        };
        let last = classes.len() - 1;
        classes.swap(position, last);
        self.selector_marker().mark(element, SelectorChange::ClassLeaving);
        self.elements[element].classes.pop();
    }

    pub fn states(&self, element: ElementId) -> States {
        self.elements[element].states
    }

    /// Turns the given states of the element on or off; its other states stay as they are.
    /// The tree turns `hover`, `active` and the focus states on and off itself as input
    /// arrives. A focused element that is disabled loses focus.
    pub fn set_state(&mut self, element: ElementId, state: States, is_on: bool) {
        self.selector_marker().set_state(element, state, is_on);
        let disables_focused = is_on && state.contains(States::DISABLED) && self.input.focused == Some(element);
        if disables_focused {
            self.blur();
        }
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
        if let Some(semantics) = &mut held.semantics {
            semantics.text_changed();
        }
        held.dirty.insert(Dirty::PAINT);
        let is_attached = held.is_attached;
        self.elements.invalidate_text(element);
        if is_attached {
            self.elements.invalidate_layout(element);
            self.has_unpainted_changes = true;
            self.request_frame();
        }
    }

    /// Shows the image in a [`Role::Image`] element. The picture takes `natural_size` CSS
    /// pixels once decoded; until the host knows it, the element shows its box alone and is
    /// laid out without a size of its own.
    ///
    /// # Panics
    ///
    /// When the element is not an image.
    pub fn set_image(&mut self, element: ElementId, image: ImageId, natural_size: Option<Size>) {
        self.set_media(element, Role::Image, MediaContent::Image { image, natural_size });
    }

    /// Shows the icon in a [`Role::Icon`] element, tinted with the `color` of the element.
    ///
    /// # Panics
    ///
    /// When the element is not an icon.
    pub fn set_icon(&mut self, element: ElementId, icon: IconId) {
        self.set_media(element, Role::Icon, MediaContent::Icon(icon));
    }

    /// Hands the content box of a [`Role::Canvas`] element to the application to draw into.
    ///
    /// # Panics
    ///
    /// When the element is not a canvas.
    pub fn set_canvas(&mut self, element: ElementId, canvas: CanvasId) {
        self.set_media(element, Role::Canvas, MediaContent::Canvas(canvas));
    }

    fn set_media(&mut self, element: ElementId, role: Role, media: MediaContent) {
        let held = &mut self.elements[element];
        assert!(held.role == role, "{:?} shows no {role:?}", held.role);
        if held.media == media {
            return;
        }
        held.media = media;
        held.dirty.insert(Dirty::PAINT);
        let is_attached = held.is_attached;
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
            on_surface = on_surface.then(&local_transform(transformed));
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

    /// The style stage of the frame shown at `now`, a time on the clock of the host: resolves
    /// the style of every element a change marked, and of the elements that inherit from them,
    /// and works out what every element in motion shows at that time. While a transition, an
    /// animation or a spring goes on, the tree asks for the next frame. Returns the number of
    /// elements whose style was resolved.
    ///
    /// # Panics
    ///
    /// When `now` is earlier than the time of the frame before it.
    pub fn resolve_styles(&mut self, now: Duration) -> usize {
        assert!(now >= self.timeline.now, "the time of a frame never goes back");
        self.timeline.now = now;
        self.frame_requests.frame_began();
        for position in 0..self.moving_elements.len() {
            let moving = self.moving_elements[position];
            if self.elements.contains(moving) && self.elements[moving].is_attached {
                self.elements.mark(moving, Dirty::MOTION, Dirty::STYLE_BELOW);
            }
        }
        self.moving_elements.clear();
        let outcome = StylePass {
            elements: &mut self.elements,
            resolver: StyleResolver {
                style_sheet: &self.style_sheet,
                theme: &self.theme,
            },
            motion: MotionResolver {
                style_sheet: &self.style_sheet,
                theme: &self.theme,
                timeline: self.timeline,
            },
            scratch: &mut self.style_scratch,
            pending: &mut self.pending,
            moving: &mut self.moving_elements,
        }
        .run(self.root);
        self.paint_order_is_stale |= outcome.reorders_paint;
        self.has_unpainted_changes |= outcome.repaints || outcome.reorders_paint;
        self.advance_springs();
        if !self.moving_elements.is_empty() || self.springs.iter().any(|spring| !spring.is_settled) {
            self.request_frame();
        }
        outcome.resolved_elements
    }

    /// Follows the setting of the system that asks for less motion. While it is on, a
    /// transition ends at once, a spring fades its content in at its target instead of moving
    /// it, and an animation that never ends does not run.
    pub fn set_reduced_motion(&mut self, reduces_motion: bool) {
        if self.timeline.reduces_motion == reduces_motion {
            return;
        }
        self.timeline.reduces_motion = reduces_motion;
        if reduces_motion {
            for spring in &mut self.springs {
                spring.is_settled = true;
            }
        }
        self.pending.clear();
        self.pending.push(self.root);
        while let Some(id) = self.pending.pop() {
            let element = &mut self.elements[id];
            element.dirty.insert(Dirty::STYLE.union(Dirty::STYLE_BELOW));
            self.pending.extend(&element.children);
        }
        self.request_frame();
    }

    /// Starts a spring that moves a value from `from` towards `to`, at `velocity` units per
    /// second to begin with. It runs from the next frame on, holding the frames of the tree
    /// until it settles; under reduced motion it fades instead.
    pub fn start_spring(&mut self, spring: Spring, from: f32, to: f32, velocity: f32) -> SpringId {
        self.last_spring += 1;
        let id = SpringId::new(self.last_spring);
        self.springs.push(RunningSpring {
            id,
            spring,
            from,
            to,
            velocity,
            start_time: None,
            fades: self.timeline.reduces_motion,
            is_settled: false,
        });
        self.request_frame();
        id
    }

    /// Where the value of the spring stands at the time of the last frame.
    ///
    /// # Panics
    ///
    /// When the spring was stopped.
    pub fn spring_sample(&self, spring: SpringId) -> SpringSample {
        self.running_spring(spring).sample(&self.timeline)
    }

    /// Stops the spring; its ID names nothing from now on.
    ///
    /// # Panics
    ///
    /// When the spring was stopped already.
    pub fn stop_spring(&mut self, spring: SpringId) {
        let position = self
            .springs
            .iter()
            .position(|running| running.id == spring)
            .expect("the spring runs until it is stopped");
        self.springs.remove(position);
    }

    fn running_spring(&self, spring: SpringId) -> &RunningSpring {
        self.springs
            .iter()
            .find(|running| running.id == spring)
            .expect("the spring runs until it is stopped")
    }

    /// Starts the springs started since the last frame at the time of this one and notes the
    /// springs that came to rest.
    fn advance_springs(&mut self) {
        for spring in &mut self.springs {
            spring.start_time.get_or_insert(self.timeline.now);
            if !spring.is_settled && spring.sample(&self.timeline).is_settled {
                spring.is_settled = true;
            }
        }
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
    /// collects the damage. Returns the number of elements whose items were rebuilt. A focused
    /// element that is no longer rendered loses focus here.
    ///
    /// # Panics
    ///
    /// When a change waits for the layout stage.
    pub fn build_display_list(&mut self) -> usize {
        assert!(
            !self.elements.layout_is_stale(),
            "layout is computed before the display list"
        );
        // Past the style stage of this frame: the elements around the lost focus are styled
        // again by the next one.
        self.focus_manager().drop_hidden_focus();
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
        if rebuilt_elements > 0 && self.input.pointer.is_some() {
            self.input.hover_is_stale = true;
            self.request_frame();
        }
        rebuilt_elements
    }

    /// The element shown at a point of the surface, as the last display list drew the tree:
    /// the topmost box that holds the point and that no clip hides there. A box is found
    /// whatever it paints, and by its border edge, rounded corners cut off.
    pub fn element_at(&mut self, point: Point) -> Option<ElementId> {
        HitTest {
            elements: &self.elements,
            steps: &self.paint_steps,
            group_points: &mut self.input.group_points,
        }
        .element_at(point)
    }

    /// Takes what the pointer did on the surface: moves hover with it, makes the element the
    /// primary button goes down on active, with its ancestors, until the button is released,
    /// gives focus to the innermost focusable element around a press, and tells the elements
    /// concerned through `handler`.
    ///
    /// From a press of the primary button to its release the pointer belongs to the pressed
    /// element: it hears of every move and of the release, and nothing outside it is hovered.
    pub fn pointer_input(&mut self, input: &PointerInput, handler: &mut dyn InputHandler) {
        let is_on_surface = input.kind != PointerInputKind::Exited;
        let hit = if is_on_surface {
            self.element_at(input.position)
        } else {
            None
        };
        self.input.pointer = is_on_surface.then_some(input.position);
        self.input.pointer_modifiers = input.modifiers;
        self.input.hover_is_stale = false;
        PointerRouter {
            targets: self.pointer_targets(),
            handler,
        }
        .route(input, hit);
        if matches!(input.kind, PointerInputKind::Pressed { .. }) {
            self.focus_manager().pointer_pressed(hit);
        }
    }

    /// Works out again what a resting pointer is over when the last frame may have moved the
    /// boxes under it, and tells the elements it left and came over. A host calls this at the
    /// start of every frame, before the style stage.
    pub fn refresh_hover(&mut self, handler: &mut dyn InputHandler) {
        if !mem::take(&mut self.input.hover_is_stale) {
            return;
        }
        let Some(position) = self.input.pointer else {
            return;
        };
        let hit = self.element_at(position);
        let resting = PointerInput {
            kind: PointerInputKind::Moved,
            position,
            modifiers: self.input.pointer_modifiers,
        };
        PointerRouter {
            targets: self.pointer_targets(),
            handler,
        }
        .hover(hit, &resting);
    }

    /// Takes a key that went down or up: offers it to the focused element and then to each
    /// element around it, tries the shortcuts on the way and those of the surface last, and
    /// moves focus on Tab and Shift-Tab when nothing used the key. Returns whether the key
    /// was used.
    pub fn key_input(&mut self, event: &KeyEvent, handler: &mut dyn InputHandler) -> bool {
        if event.shows_keyboard_use() {
            self.focus_manager().set_modality(InputModality::Keyboard);
        }
        let router = KeyRouter {
            elements: &self.elements,
            shortcuts: &self.input.shortcuts,
            handler,
        };
        if router.route(self.input.focused.unwrap_or(self.root), event) {
            return true;
        }
        event
            .focus_move()
            .is_some_and(|direction| self.focus_manager().move_focus(direction))
    }

    pub fn focusability(&self, element: ElementId) -> Focusability {
        self.elements[element].focusability
    }

    /// Says whether the element takes focus and whether the Tab key stops on it. A focused
    /// element that stops being focusable loses focus.
    pub fn set_focusability(&mut self, element: ElementId, focusability: Focusability) {
        self.elements[element].focusability = focusability;
        if focusability == Focusability::NotFocusable && self.input.focused == Some(element) {
            self.blur();
        }
    }

    /// The element that has keyboard focus.
    pub fn focused(&self) -> Option<ElementId> {
        self.input.focused
    }

    /// Gives keyboard focus to the element. Returns whether it has focus now: an element that
    /// is outside the tree, is not focusable, is disabled or lies outside the innermost focus
    /// trap takes none.
    pub fn focus(&mut self, element: ElementId) -> bool {
        self.focus_manager().focus(element)
    }

    /// Takes keyboard focus from the element that has it.
    pub fn blur(&mut self) {
        self.focus_manager().blur();
    }

    /// Keeps focus inside the element until the trap is released: the Tab key walks the stops
    /// inside it, and nothing outside it takes focus. Focus that is outside it now is dropped.
    /// Of several traps the one set last holds.
    ///
    /// # Panics
    ///
    /// When the element is outside the tree.
    pub fn trap_focus(&mut self, element: ElementId) {
        assert!(
            self.elements[element].is_attached,
            "a focus trap is set inside the tree"
        );
        self.focus_manager().trap(element);
    }

    /// Releases the trap set on the element. When it was the trap set last, the element that
    /// had focus when it was set takes focus back, if it still can.
    ///
    /// # Panics
    ///
    /// When the element traps no focus.
    pub fn release_focus_trap(&mut self, element: ElementId) {
        self.focus_manager().release_trap(element);
    }

    /// Registers a shortcut: when the chord is pressed with focus in the scope and no element
    /// between focus and the scope uses the key, the handler of the input hears of it. Of two
    /// shortcuts with one chord the innermost scope answers, and in one scope the one
    /// registered last. The shortcuts of an element go with it when it is removed.
    pub fn register_shortcut(&mut self, scope: ShortcutScope, chord: KeyChord) -> ShortcutId {
        self.input.last_shortcut += 1;
        let id = ShortcutId::new(self.input.last_shortcut);
        self.input.shortcuts.push(Shortcut { id, scope, chord });
        id
    }

    /// # Panics
    ///
    /// When the shortcut is not registered.
    pub fn unregister_shortcut(&mut self, shortcut: ShortcutId) {
        let position = self
            .input
            .shortcuts
            .iter()
            .position(|registered| registered.id == shortcut)
            .expect("a shortcut is registered until it is unregistered or its element is removed");
        self.input.shortcuts.remove(position);
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
            element.is_rendered = false;
            element.containing_block = None;
            if let Some(motion) = &mut element.motion {
                motion.clear();
            }
            self.pending.extend(&element.children);
        }
    }

    fn mark_style(&mut self, element: ElementId) {
        if self.elements[element].is_attached {
            self.elements.mark(element, Dirty::STYLE, Dirty::STYLE_BELOW);
            self.request_frame();
        }
    }

    /// Asks the host for a frame although nothing of the tree changed yet: for what the host
    /// does at the start of a frame, before the stages of the tree, and what may change the tree
    /// there.
    pub fn request_frame(&mut self) {
        self.frame_requests.request();
    }

    /// The role of the node the element has in the accessibility tree.
    pub fn accessible_role(&self, element: ElementId) -> Option<accesskit::Role> {
        self.elements[element].semantics.as_deref().map(Semantics::role)
    }

    /// Gives the element a node of the role in the accessibility tree, or takes its node away
    /// with the name, the value and the actions of the node. The nodes of what is inside an
    /// element without a node belong to the nearest element around it that has one. The root
    /// is a window unless given another role.
    ///
    /// # Panics
    ///
    /// When the node of the root is taken away.
    pub fn set_accessible_role(&mut self, element: ElementId, role: Option<accesskit::Role>) {
        assert!(
            role.is_some() || element != self.root,
            "the root always has a node in the accessibility tree"
        );
        let held = &mut self.elements[element].semantics;
        let changes = match (held.as_deref_mut(), role) {
            (Some(semantics), Some(role)) => semantics.set_role(role),
            (None, Some(role)) => {
                *held = Some(Box::new(Semantics::new(role)));
                true
            }
            (Some(_), None) => {
                *held = None;
                true
            }
            (None, None) => false,
        };
        self.accessibility_changed(element, changes);
    }

    /// Names the node of the element: what an assistive technology calls it.
    ///
    /// # Panics
    ///
    /// When the element has no accessible role.
    pub fn set_accessible_name(&mut self, element: ElementId, name: &str) {
        let changes = self.semantics_mut(element).set_name(name);
        self.accessibility_changed(element, changes);
    }

    /// Says what the node of the element holds. With no value of its own a text element
    /// gives its text.
    ///
    /// # Panics
    ///
    /// When the element has no accessible role.
    pub fn set_accessible_value(&mut self, element: ElementId, value: AccessibleValue<'_>) {
        let changes = self.semantics_mut(element).set_value(value);
        self.accessibility_changed(element, changes);
    }

    /// Says what an assistive technology may ask the element to do. Taking focus is offered
    /// by every element that takes it.
    ///
    /// # Panics
    ///
    /// When the element has no accessible role.
    pub fn set_accessible_actions(&mut self, element: ElementId, actions: AccessibleActions) {
        let changes = self.semantics_mut(element).set_actions(actions);
        self.accessibility_changed(element, changes);
    }

    /// The whole accessibility tree as the last frame left it: what an assistive technology
    /// is given when it starts to read the surface. The updates that follow tell what changed
    /// since.
    pub fn accessibility_tree(&mut self) -> TreeUpdate {
        self.accessibility_pass(true)
    }

    /// What changed of the accessibility tree since the last update, or since the whole tree
    /// was given: the nodes that are new or differ, and the node that has focus. A host works
    /// this out after the display list stage of a frame, for as long as an assistive
    /// technology reads the surface.
    pub fn accessibility_update(&mut self) -> TreeUpdate {
        self.accessibility_pass(false)
    }

    /// Takes what an assistive technology asks of a node: gives focus to the element when
    /// asked to, as the keyboard would, and tells the element through `handler` of an action
    /// it offers. A request the element does not offer, or one for a node that is no longer
    /// in the tree, does nothing.
    pub fn accessibility_action(&mut self, request: &ActionRequest, handler: &mut dyn InputHandler) {
        let target = element_of(request.target_node);
        if !self.elements.contains(target) {
            return;
        }
        let element = &self.elements[target];
        let Some(semantics) = element.semantics.as_deref() else {
            return;
        };
        if !element.is_attached || element.states.contains(States::DISABLED) {
            return;
        }
        let offered = semantics.actions();
        let action = match (request.action, &request.data) {
            (Action::Focus, _) => {
                let mut focus = self.focus_manager();
                focus.set_modality(InputModality::Keyboard);
                focus.focus(target);
                return;
            }
            (Action::Click, _) if offered.contains(AccessibleActions::CLICK) => AccessibilityAction::Click,
            (Action::Increment, _) if offered.contains(AccessibleActions::INCREMENT) => AccessibilityAction::Increment,
            (Action::Decrement, _) if offered.contains(AccessibleActions::DECREMENT) => AccessibilityAction::Decrement,
            (Action::SetValue, Some(ActionData::Value(text))) if offered.contains(AccessibleActions::SET_VALUE) => {
                AccessibilityAction::SetText(text)
            }
            (Action::SetValue, Some(ActionData::NumericValue(number)))
                if offered.contains(AccessibleActions::SET_VALUE) =>
            {
                AccessibilityAction::SetNumber(*number)
            }
            _ => return,
        };
        handler.accessibility_action(target, &action);
    }

    fn accessibility_pass(&mut self, holds_whole_tree: bool) -> TreeUpdate {
        self.accessibility_updates += 1;
        AccessibilityPass {
            elements: &mut self.elements,
            root: self.root,
            focused: self.input.focused,
            scale: self.scale,
            update: self.accessibility_updates,
            holds_whole_tree,
            nodes: Vec::new(),
        }
        .run()
    }

    fn semantics_mut(&mut self, element: ElementId) -> &mut Semantics {
        self.elements[element]
            .semantics
            .as_deref_mut()
            .expect("an element gets an accessible role before a name, a value or actions")
    }

    /// A change to the node of an attached element waits for the frame whose update tells of it.
    fn accessibility_changed(&mut self, element: ElementId, changes: bool) {
        if changes && self.elements[element].is_attached {
            self.request_frame();
        }
    }

    fn pointer_targets(&mut self) -> PointerTargets<'_> {
        PointerTargets {
            marker: SelectorMarker {
                elements: &mut self.elements,
                resolver: StyleResolver {
                    style_sheet: &self.style_sheet,
                    theme: &self.theme,
                },
                frame_requests: &mut self.frame_requests,
            },
            input: &mut self.input,
        }
    }

    fn focus_manager(&mut self) -> FocusManager<'_> {
        FocusManager {
            marker: SelectorMarker {
                elements: &mut self.elements,
                resolver: StyleResolver {
                    style_sheet: &self.style_sheet,
                    theme: &self.theme,
                },
                frame_requests: &mut self.frame_requests,
            },
            input: &mut self.input,
            root: self.root,
        }
    }

    fn selector_marker(&mut self) -> SelectorMarker<'_> {
        SelectorMarker {
            elements: &mut self.elements,
            resolver: StyleResolver {
                style_sheet: &self.style_sheet,
                theme: &self.theme,
            },
            frame_requests: &mut self.frame_requests,
        }
    }
}
