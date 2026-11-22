use morok_core::{DragOutcome, Point, PointerButton, PointerInput, PointerInputKind};
use morok_reactive::Runtime;

use super::overlays::is_inside;
use crate::view::carried_drag::CarriedDrag;
use crate::view::surface_id::SurfaceId;
use crate::view::surfaces::Surfaces;

/// The drags of the pointer across the surfaces of the application. A drag of a component
/// lives in the tree of its surface for as long as the pointer carries it. While the pointer is
/// over another surface, that tree lets go of its own drop targets and the other surface takes
/// the drag as one that began outside it, in its own coordinates; the tree of the source ends
/// the drag with the outcome the other surface gives. An overlay is a surface like a window,
/// and the room it leaves around its panel belongs to the surface under it.
impl Surfaces {
    /// Takes what the pointer did while a press on `surface` holds it and it is over `under`,
    /// another surface of the application and the topmost one there, at `position` in CSS
    /// pixels of `under`. A drag the pointer carries from `surface` goes over to `under`;
    /// whatever else holds the press hears of the pointer as usual, in the coordinates of
    /// `surface`.
    pub fn pointer_input_over(
        &mut self,
        runtime: &mut Runtime<Self>,
        surface: SurfaceId,
        input: &PointerInput,
        under: SurfaceId,
        position: Point,
    ) {
        if self.surface(surface).tree.pointer_drag().is_none() {
            self.pointer_input(runtime, surface, input);
            return;
        }
        self.drag_pointer(runtime, surface, input, under, position);
    }

    /// Carries the drag of `source` where the pointer is: at `position` of `under`, which is
    /// `input.position` when `under` is the source. Over its own surface the tree of the source
    /// follows the pointer itself; a release over no surface of the application cancels the
    /// drag there.
    pub(super) fn drag_pointer(
        &mut self,
        runtime: &mut Runtime<Self>,
        source: SurfaceId,
        input: &PointerInput,
        under: SurfaceId,
        position: Point,
    ) {
        let (over, at) = self.surface_taking_pointer(under, position);
        if over == source {
            self.leave_carried_surface(runtime);
            let on_source = PointerInput { position: at, ..*input };
            self.surface_mut(source).pointer_input(runtime, &on_source);
            return;
        }
        match input.kind {
            PointerInputKind::Moved => self.carry_drag(runtime, source, input, over, at),
            PointerInputKind::Released {
                button: PointerButton::Primary,
                ..
            } => self.drop_carried_drag(runtime, source, input, over, at),
            PointerInputKind::Exited => {
                self.leave_carried_surface(runtime);
                self.surface_mut(source).pointer_input(runtime, input);
            }
            PointerInputKind::Pressed { .. } | PointerInputKind::Released { .. } => {
                self.surface_mut(source).pointer_input(runtime, input);
            }
        }
    }

    /// The surface of the drag the pointer carries from a component of it, if the pointer
    /// carries one.
    pub(super) fn pointer_drag_source(&self) -> Option<SurfaceId> {
        self.slots.iter().enumerate().find_map(|(index, slot)| {
            let held = slot.surface.as_ref()?;
            held.tree.pointer_drag()?;
            let index = u32::try_from(index).expect("an application opens fewer than 2^32 surfaces");
            Some(SurfaceId::new(index, slot.generation))
        })
    }

    /// The surface the drag is carried over leaves it: its drop target hears that it left.
    pub(super) fn leave_carried_surface(&mut self, runtime: &mut Runtime<Self>) {
        if let Some(carried) = self.carried_drag.take()
            && self.contains(carried.over)
        {
            self.surface_mut(carried.over).system_drag_exited(runtime);
        }
    }

    /// The surface is about to close: a drag carried from it leaves the surface it is over,
    /// and a drag carried over it is carried no longer.
    pub(super) fn forget_carried_drag_of(&mut self, runtime: &mut Runtime<Self>, closed: SurfaceId) {
        let Some(carried) = self.carried_drag else {
            return;
        };
        if carried.over == closed {
            self.carried_drag = None;
        } else if carried.source == closed {
            self.leave_carried_surface(runtime);
        }
    }

    /// The drag moved over `over`, at `at` in its coordinates: the surface takes it, or
    /// follows it on. The tree of the source lets go of its own target once the drag leaves
    /// its surface.
    fn carry_drag(
        &mut self,
        runtime: &mut Runtime<Self>,
        source: SurfaceId,
        input: &PointerInput,
        over: SurfaceId,
        at: Point,
    ) {
        let carried = CarriedDrag { source, over };
        if self.carried_drag == Some(carried) {
            self.surface_mut(over).system_drag_moved(runtime, at, input.modifiers);
            return;
        }
        self.leave_carried_surface(runtime);
        let let_go = PointerInput {
            kind: PointerInputKind::Exited,
            ..*input
        };
        self.surface_mut(source).pointer_input(runtime, &let_go);
        let payload = self
            .surface(source)
            .tree
            .pointer_drag()
            .cloned()
            .expect("the pointer carries a drag from the source");
        self.surface_mut(over)
            .system_drag_entered(runtime, payload, at, input.modifiers);
        self.carried_drag = Some(carried);
    }

    /// The primary button came up over `over`: the drag is dropped there, and the tree of the
    /// source ends it as dropped when a target took it, and as cancelled when none did.
    fn drop_carried_drag(
        &mut self,
        runtime: &mut Runtime<Self>,
        source: SurfaceId,
        input: &PointerInput,
        over: SurfaceId,
        at: Point,
    ) {
        if self.carried_drag != Some(CarriedDrag { source, over }) {
            self.carry_drag(runtime, source, input, over, at);
        }
        let was_taken = self.surface_mut(over).system_drag_dropped(runtime, at, input.modifiers);
        self.carried_drag = None;
        let outcome = if was_taken {
            DragOutcome::Dropped
        } else {
            DragOutcome::Cancelled
        };
        self.surface_mut(source).end_pointer_drag(runtime, outcome);
    }

    /// The surface that takes the pointer at `position` of `under`, with the place of the
    /// pointer on it: the room an overlay leaves around its panel belongs to the surface the
    /// overlay is opened over. An overlay the host has not placed yet keeps the pointer.
    fn surface_taking_pointer(&mut self, under: SurfaceId, position: Point) -> (SurfaceId, Point) {
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
}
