use morok_core::{
    ElementId, ElementTree, KeyEvent, PlacedOverlay, Point, PointerInput, PointerInputKind, Rect, Role, ScrollInput,
};
use morok_reactive::Runtime;
use morok_style::States;

use crate::styles::CLASSES;
use crate::view::mount::Mount;
use crate::view::overlay::Overlay;
use crate::view::overlay_anchor::OverlayAnchor;
use crate::view::overlay_attachment::OverlayAttachment;
use crate::view::overlay_geometry::OverlayGeometry;
use crate::view::overlay_place::OverlayPlace;
use crate::view::surface::Surface;
use crate::view::surface_id::SurfaceId;
use crate::view::surfaces::Surfaces;

const NOT_AN_OVERLAY: &str = "the surface is an overlay";

/// The overlays of an application: surfaces opened over other surfaces. The surfaces decide
/// what the pointer and the keyboard mean for an overlay; a host shows each in a window of
/// its own and gives it the input of that window.
impl Surfaces {
    /// Opens an overlay over `parent`: a surface of its own, styled as `parent` is, that shows
    /// the element `content` builds with room around it for its shadow. `content` is handed
    /// where the host places the overlay: the side of the anchor, which changes when the
    /// overlay flips, and how far the edges of the screen move it along that side. The overlay
    /// belongs to `owner`, an element of `parent`: it is closed once that element is removed,
    /// and when `parent` is closed. The host of `parent` shows it after the next frame of
    /// `parent`.
    ///
    /// # Panics
    ///
    /// When `parent` was closed, and when its style sheet was not compiled with the
    /// stylesheets of the kit first.
    pub fn open_overlay(
        &mut self,
        runtime: &mut Runtime<Self>,
        parent: SurfaceId,
        owner: ElementId,
        overlay: Overlay,
        content: impl FnOnce(&mut Mount<'_>, OverlayPlace) -> ElementId,
    ) -> SurfaceId {
        let opened_over = self.surface_mut(parent);
        opened_over.tree.request_frame();
        let style_sheet = opened_over.tree.style_sheet().clone();
        let theme = opened_over.tree.theme().clone();
        let scope = runtime.scope_in(opened_over.scope, |_| {});
        let opened = self.open_in_scope(scope, style_sheet, theme);
        let tree = &mut self.surface_mut(opened).tree;
        let frame = tree.create(Role::Element);
        tree.add_class(frame, CLASSES.overlay_surface);
        tree.append_child(tree.root(), frame);
        let mut placed = None;
        let panel = self.mount(runtime, opened, frame, |mount| {
            let place = OverlayPlace {
                side: mount.runtime().signal(overlay.placement.side),
                edge_shift: mount.runtime().signal(0.0),
            };
            placed = Some(place);
            content(mount, place)
        });
        self.surface_mut(opened).overlay = Some(OverlayAttachment {
            parent,
            owner,
            anchor: overlay.anchor,
            placement: overlay.placement,
            interaction: overlay.interaction,
            frame,
            panel,
            anchor_box: None,
            place: placed.expect("mounting calls the component"),
            origin: None,
            passes_pointer_on: false,
        });
        opened
    }

    /// Lists the overlays opened over the surface into `overlays`, in place of what it held.
    pub fn overlays_of(&self, surface: SurfaceId, overlays: &mut Vec<SurfaceId>) {
        overlays.clear();
        overlays.extend((0..self.slots.len()).filter_map(|index| self.overlay_at(index, surface)));
    }

    /// The surface the overlay is opened over; none for the surface of a window.
    pub fn parent_of(&self, surface: SurfaceId) -> Option<SurfaceId> {
        self.surface(surface)
            .overlay
            .as_ref()
            .map(|attachment| attachment.parent)
    }

    /// Brings the overlays opened over the surface in line with the frame the surface just
    /// ran: each is placed against the box its anchor has now, and an overlay whose owner was
    /// removed from the tree is closed. A host calls this after every frame of a surface,
    /// before it reads the geometry of the overlays.
    ///
    /// # Panics
    ///
    /// When a change of the surface waits for the style stage.
    pub fn follow_overlay_anchors(&mut self, runtime: &mut Runtime<Self>, surface: SurfaceId) {
        for index in 0..self.slots.len() {
            let Some(overlay) = self.overlay_at(index, surface) else {
                continue;
            };
            let attachment = self.attachment(overlay);
            let (owner, anchor) = (attachment.owner, attachment.anchor);
            let tree = &self.surface(surface).tree;
            if !tree.contains(owner) {
                self.close(runtime, overlay);
                continue;
            }
            let anchor_box = match anchor {
                OverlayAnchor::Element(element) => tree.contains(element).then(|| tree.surface_box(element)),
                OverlayAnchor::Point(point) => Some(Rect::new(point.x, point.y, 0.0, 0.0)),
            };
            self.attachment_mut(overlay).anchor_box = anchor_box;
        }
    }

    /// Where the host shows the overlay, as the last frames of the overlay and of the surface
    /// it is opened over left it; none while its anchor has no box to place it against.
    ///
    /// # Panics
    ///
    /// When the surface is no overlay.
    pub fn overlay_geometry(&self, overlay: SurfaceId) -> Option<OverlayGeometry> {
        let surface = self.surface(overlay);
        let attachment = surface.overlay.as_ref().expect(NOT_AN_OVERLAY);
        Some(OverlayGeometry {
            anchor: attachment.anchor_box?,
            placement: attachment.placement,
            size: surface.tree.border_box(attachment.frame).size(),
            panel: surface.tree.border_box(attachment.panel),
            input: attachment.interaction.input(),
        })
    }

    /// The host placed the overlay: `placed` holds the rectangle of its surface in CSS pixels
    /// of the surface it is opened over, the side of its anchor it lies on, and how far the
    /// edges of the screen moved it along that side.
    ///
    /// # Panics
    ///
    /// When the surface is no overlay.
    pub fn overlay_placed(&mut self, runtime: &mut Runtime<Self>, overlay: SurfaceId, placed: PlacedOverlay) {
        let attachment = self.attachment_mut(overlay);
        attachment.origin = Some(placed.frame.origin());
        let place = attachment.place;
        if runtime.untracked(|runtime| runtime.get(place.side)) != placed.side {
            runtime.set(place.side, placed.side);
        }
        if runtime.untracked(|runtime| runtime.get(place.edge_shift)).to_bits() != placed.edge_shift.to_bits() {
            runtime.set(place.edge_shift, placed.edge_shift);
        }
    }

    /// Takes what the pointer did on a surface, as the host of that surface saw it, and lets
    /// the components hear of it.
    ///
    /// Over the room an overlay leaves around its panel the pointer belongs to the surface
    /// under the overlay, which hears of it in its own coordinates. A press dismisses every
    /// overlay it lands outside of: an overlay keeps the presses on its panel, on the overlays
    /// opened over it and on the element it is anchored to. A drag the pointer carries from
    /// a component of the surface goes wherever the pointer goes.
    pub fn pointer_input(&mut self, runtime: &mut Runtime<Self>, surface: SurfaceId, input: &PointerInput) {
        if self.surface(surface).tree.pointer_drag().is_some() {
            self.drag_pointer(runtime, surface, input, surface, input.position);
            return;
        }
        let held = self.surface_mut(surface);
        let Some(attachment) = &held.overlay else {
            self.deliver_pointer(runtime, surface, input);
            return;
        };
        let (parent, panel, origin) = (attachment.parent, attachment.panel, attachment.origin);
        let passed_on_before = attachment.passes_pointer_on;
        let left = PointerInput {
            kind: PointerInputKind::Exited,
            ..*input
        };
        let is_on_surface = input.kind != PointerInputKind::Exited;
        let is_on_panel = is_on_surface && pointer_belongs_to(&mut held.tree, panel, input.position);
        let passes_on = is_on_surface && !is_on_panel;
        self.attachment_mut(surface).passes_pointer_on = passes_on;
        if passed_on_before && !passes_on {
            self.pointer_input(runtime, parent, &left);
        }
        if !passed_on_before && passes_on {
            self.deliver_pointer(runtime, surface, &left);
        }
        if !passes_on {
            if is_on_surface || !passed_on_before {
                self.deliver_pointer(runtime, surface, input);
            }
            return;
        }
        if let Some(origin) = origin {
            let under = PointerInput {
                position: Point {
                    x: input.position.x + origin.x,
                    y: input.position.y + origin.y,
                },
                ..*input
            };
            self.pointer_input(runtime, parent, &under);
        } else if matches!(input.kind, PointerInputKind::Pressed { .. }) {
            self.dismiss(runtime, surface);
        }
    }

    /// Takes a scroll of the wheel or the trackpad over a surface, as the host of that surface
    /// saw it. Over the room an overlay leaves around its panel the scroll belongs to the
    /// surface under the overlay, which scrolls in its own coordinates.
    pub fn scroll_input(&mut self, surface: SurfaceId, input: &ScrollInput) {
        let (under, position) = self.surface_taking_pointer(surface, input.position);
        let scrolled = ScrollInput { position, ..*input };
        self.surface_mut(under).scroll_input(&scrolled);
    }

    /// The surface that takes the pointer at `position` of `under`, with the place of the
    /// pointer on it: the room an overlay leaves around its panel belongs to the surface the
    /// overlay is opened over. An overlay the host has not placed yet keeps the pointer.
    pub(super) fn surface_taking_pointer(&mut self, under: SurfaceId, position: Point) -> (SurfaceId, Point) {
        let (mut surface, mut at) = (under, position);
        while let Some(attachment) = &self.surface(surface).overlay {
            let (parent, panel) = (attachment.parent, attachment.panel);
            let Some(origin) = attachment.origin else {
                break;
            };
            let tree = &mut self.surface_mut(surface).tree;
            if tree.element_at(at).is_some_and(|hit| is_inside(tree, panel, hit)) {
                break;
            }
            at = Point {
                x: at.x + origin.x,
                y: at.y + origin.y,
            };
            surface = parent;
        }
        (surface, at)
    }

    /// Takes a key that went down or up on the surface that has the keyboard. Returns whether
    /// it was used. Escape cancels the drag the pointer carries, over whichever surface it is,
    /// and does nothing else; Escape that nothing on an overlay used dismisses an overlay that
    /// takes the keyboard.
    pub fn key_input(&mut self, runtime: &mut Runtime<Self>, surface: SurfaceId, event: &KeyEvent) -> bool {
        if event.is_plain_escape()
            && let Some(source) = self.pointer_drag_source()
        {
            self.leave_carried_surface(runtime);
            return self.surface_mut(source).key_input(runtime, event);
        }
        if self.surface_mut(surface).key_input(runtime, event) {
            return true;
        }
        if event.is_plain_escape() && self.takes_keyboard(surface) {
            self.dismiss(runtime, surface);
            return true;
        }
        false
    }

    /// The user turned away from the surfaces of the application: pressed outside all of
    /// them, or went to another application. Every overlay that takes the pointer is
    /// dismissed.
    pub fn dismiss_overlays(&mut self, runtime: &mut Runtime<Self>) {
        for index in 0..self.slots.len() {
            if let Some(overlay) = self.overlay_in_slot(index) {
                self.dismiss(runtime, overlay);
            }
        }
    }

    /// An overlay opened over the surface, if any is.
    pub(super) fn first_overlay_of(&self, surface: SurfaceId) -> Option<SurfaceId> {
        (0..self.slots.len()).find_map(|index| self.overlay_at(index, surface))
    }

    /// The pointer input reached the surface it is meant for.
    fn deliver_pointer(&mut self, runtime: &mut Runtime<Self>, surface: SurfaceId, input: &PointerInput) {
        if matches!(input.kind, PointerInputKind::Pressed { .. }) {
            self.dismiss_overlays_pressed_outside(runtime, surface, input.position);
        }
        self.surface_mut(surface).pointer_input(runtime, input);
    }

    fn dismiss_overlays_pressed_outside(&mut self, runtime: &mut Runtime<Self>, pressed: SurfaceId, position: Point) {
        let hit = self.surface_mut(pressed).tree.element_at(position);
        for index in 0..self.slots.len() {
            if let Some(overlay) = self.overlay_in_slot(index)
                && !self.keeps_press(overlay, pressed, hit)
            {
                self.dismiss(runtime, overlay);
            }
        }
    }

    /// Whether a press on the element `hit` of the surface `pressed` leaves the overlay open.
    fn keeps_press(&self, overlay: SurfaceId, pressed: SurfaceId, hit: Option<ElementId>) -> bool {
        let mut inside = Some(pressed);
        while let Some(surface) = inside {
            if surface == overlay {
                return true;
            }
            inside = self.parent_of(surface);
        }
        let attachment = self.attachment(overlay);
        let OverlayAnchor::Element(anchor) = attachment.anchor else {
            return false;
        };
        attachment.parent == pressed && hit.is_some_and(|hit| is_inside(&self.surface(pressed).tree, anchor, hit))
    }

    /// Whether the surface is an overlay that takes the keyboard.
    fn takes_keyboard(&self, surface: SurfaceId) -> bool {
        self.surface(surface)
            .overlay
            .as_ref()
            .is_some_and(|attachment| attachment.interaction.input().takes_keyboard())
    }

    /// Asks an overlay that takes the pointer to go; one that only shows stays.
    fn dismiss(&mut self, runtime: &mut Runtime<Self>, overlay: SurfaceId) {
        if let Some(on_dismiss) = self.attachment_mut(overlay).interaction.on_dismiss() {
            on_dismiss(runtime);
        }
    }

    /// The overlay in the slot, when the slot holds one that is opened over `parent`.
    fn overlay_at(&self, index: usize, parent: SurfaceId) -> Option<SurfaceId> {
        self.overlay_in_slot(index)
            .filter(|overlay| self.attachment(*overlay).parent == parent)
    }

    /// The overlay in the slot, when the slot holds one.
    fn overlay_in_slot(&self, index: usize) -> Option<SurfaceId> {
        let slot = &self.slots[index];
        let held: &Surface = slot.surface.as_ref()?;
        held.overlay.as_ref()?;
        let index = u32::try_from(index).expect("an application opens fewer than 2^32 surfaces");
        Some(SurfaceId::new(index, slot.generation))
    }

    fn attachment(&self, overlay: SurfaceId) -> &OverlayAttachment {
        self.surface(overlay).overlay.as_ref().expect(NOT_AN_OVERLAY)
    }

    fn attachment_mut(&mut self, overlay: SurfaceId) -> &mut OverlayAttachment {
        self.surface_mut(overlay).overlay.as_mut().expect(NOT_AN_OVERLAY)
    }
}

/// Whether the pointer at `position` is the panel's: over it, or held by a press that began
/// on it.
fn pointer_belongs_to(tree: &mut ElementTree, panel: ElementId, position: Point) -> bool {
    tree.states(panel).contains(States::ACTIVE)
        || tree.element_at(position).is_some_and(|hit| is_inside(tree, panel, hit))
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
