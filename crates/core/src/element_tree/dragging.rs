use super::ElementTree;
use crate::drag::drag_outcome::DragOutcome;
use crate::drag::drag_reach::DragReach;
use crate::drag::drag_router::DragRouter;
use crate::drag::drag_session::DragSession;
use crate::drag::system_drag::SystemDrag;
use crate::element_id::ElementId;
use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::input::input_handler::InputHandler;
use crate::input::modifiers::Modifiers;
use crate::input::pointer_input::PointerInput;
use crate::input::pointer_input_kind::PointerInputKind;
use crate::input::pointer_router::PointerRouter;
use crate::paint::display_list::DisplayList;
use crate::paint::subtree_picture::SubtreePicture;
use crate::transfer::payload::Payload;
use crate::transfer::payload_type::PayloadType;

/// Drag and drop: the drag sources and the drop targets of the tree, the drag a source hands
/// to the system, and the drags that began outside the tree — of the system, or of another
/// surface of the application — that the host of the tree carries over its surface.
impl ElementTree {
    pub fn is_drag_source(&self, element: ElementId) -> bool {
        self.elements[element].is_drag_source()
    }

    /// Says whether the pointer may drag the element. A press of the primary button on a drag
    /// source, or inside it on nothing that uses the press, becomes a drag once the pointer
    /// has travelled a few pixels: the handler of the input says what the drag carries and how
    /// far it goes, the press ends without a click, and the source hears how the drag ended.
    /// A disabled source is not dragged.
    pub fn set_drag_source(&mut self, element: ElementId, is_drag_source: bool) {
        if self.elements[element].is_drag_source() == is_drag_source {
            return;
        }
        self.elements[element].drag_role.get_or_insert_default().is_source = is_drag_source;
    }

    /// The payload types a drag is dropped on the element in; none unless it is a drop target.
    pub fn drop_types(&self, element: ElementId) -> &[PayloadType] {
        self.elements[element].drop_types()
    }

    /// Makes the element a drop target of the drags that carry any of the types; with no
    /// types it is a drop target no longer. A drag is offered to the innermost drop target
    /// under it that takes a type it carries: that target hears what the drag does through
    /// the handler of the input, and while the drag is over it, it is hovered, with its
    /// ancestors, and nothing else is. A disabled target takes no drag.
    pub fn set_drop_types(&mut self, element: ElementId, types: &[PayloadType]) {
        if self.elements[element].drop_types() == types {
            return;
        }
        let role = self.elements[element].drag_role.get_or_insert_default();
        self.accepted_drop_types.remove(&role.drop_types);
        role.drop_types = types.into();
        self.accepted_drop_types.add(types);
    }

    /// The payload types the drop targets of the tree take between them: what the host of the
    /// tree asks the system to offer its surface.
    pub fn accepted_drop_types(&self) -> Vec<PayloadType> {
        self.accepted_drop_types.types().cloned().collect()
    }

    /// How many times [`accepted_drop_types`](Self::accepted_drop_types) changed.
    pub fn drop_types_revision(&self) -> u64 {
        self.accepted_drop_types.revision()
    }

    /// What the drag under way over the surface carries; none while there is no drag.
    pub fn dragged_payload(&self) -> Option<&Payload> {
        self.input.drag.as_ref().map(|drag| &drag.payload)
    }

    /// The drag a source handed to the system since the host took the last one. The host
    /// asks after every pointer input, begins the drag in the system and tells how it ended
    /// through [`system_drag_ended`](Self::system_drag_ended).
    pub fn take_system_drag(&mut self) -> Option<SystemDrag> {
        self.input.system_drag.take()
    }

    /// The drag the host took for the system is over: its source hears how it ended, unless
    /// it left the tree meanwhile.
    pub fn system_drag_ended(&self, source: ElementId, outcome: DragOutcome, handler: &mut dyn InputHandler) {
        if self.elements.contains(source) {
            handler.drag_ended(source, outcome);
        }
    }

    /// A drag that began outside the tree came over the surface at `position`, carrying
    /// `payload`: a drag of the system, or of another surface of the application, which the
    /// host carries over this one. The pointer hovers nothing until the drag is over. Returns
    /// whether a drop target takes the drag there.
    pub fn system_drag_entered(
        &mut self,
        payload: Payload,
        position: Point,
        modifiers: Modifiers,
        handler: &mut dyn InputHandler,
    ) -> bool {
        // A drag whose end the tree was never told of gives way to the one that came.
        self.system_drag_exited_any(handler);
        let (hit, input) = self.follow_system_drag(position, modifiers);
        let mut pointer = PointerRouter {
            targets: self.pointer_targets(),
            handler,
        };
        let mut drag = DragRouter { pointer: &mut pointer };
        drag.begin(
            DragSession {
                payload,
                reach: DragReach::System,
                source: None,
            },
            &input,
        );
        drag.move_to(hit, &input)
    }

    /// The drag that began outside the tree moved to `position`. Returns whether a drop target
    /// takes it there; nothing takes a drag the tree was not told came over the surface.
    pub fn system_drag_moved(&mut self, position: Point, modifiers: Modifiers, handler: &mut dyn InputHandler) -> bool {
        if !self.follows_system_drag() {
            return false;
        }
        let (hit, input) = self.follow_system_drag(position, modifiers);
        let mut pointer = PointerRouter {
            targets: self.pointer_targets(),
            handler,
        };
        DragRouter { pointer: &mut pointer }.move_to(hit, &input)
    }

    /// The drag that began outside the tree left the surface, or ended somewhere else.
    pub fn system_drag_exited(&mut self, handler: &mut dyn InputHandler) {
        if self.follows_system_drag() {
            self.system_drag_exited_any(handler);
        }
    }

    /// The drag that began outside the tree was dropped at `position`. Returns whether a drop
    /// target took it; nothing takes a drag the tree was not told came over the surface.
    pub fn system_drag_dropped(
        &mut self,
        position: Point,
        modifiers: Modifiers,
        handler: &mut dyn InputHandler,
    ) -> bool {
        if !self.follows_system_drag() {
            return false;
        }
        let (hit, input) = self.follow_system_drag(position, modifiers);
        let mut pointer = PointerRouter {
            targets: self.pointer_targets(),
            handler,
        };
        let was_taken = DragRouter { pointer: &mut pointer }.drop_at(hit, &input);
        self.input.pointer = None;
        was_taken
    }

    /// Writes to `picture` what the last display list drew of the element and of everything
    /// inside it, and nothing else: the items in their paint order, inside the groups that
    /// place them on the surface, so that a renderer draws them where the surface shows them,
    /// and inside the clips of the elements of the subtree. The clips of the elements around
    /// it do not hold: the picture shows the whole subtree, the part a scrolled viewport hides
    /// included. Returns the rectangle of the surface they cover; none when the display list
    /// drew nothing of them.
    pub fn subtree_display_list(&self, subtree: ElementId, picture: &mut DisplayList) -> Option<Rect> {
        SubtreePicture {
            elements: &self.elements,
            display_list: &self.display_list,
            clip_owners: &self.clip_owners,
        }
        .write(subtree, picture)
    }

    /// What the drag the pointer carries from a source of the tree carries; none while the
    /// pointer drags nothing. A drag that began outside the tree is not the pointer's here.
    pub fn pointer_drag(&self) -> Option<&Payload> {
        self.input
            .drag
            .as_ref()
            .filter(|drag| drag.reach == DragReach::Surface)
            .map(|drag| &drag.payload)
    }

    /// The drag the pointer carries from a source of the tree ended over another surface of
    /// the application: dropped there, or cancelled, as `outcome` tells. The target it is over
    /// here hears that it left, its source how it ended, and the pointer, which is over the
    /// other surface, hovers nothing here.
    ///
    /// # Panics
    ///
    /// When the pointer carries no drag from a source of the tree.
    pub fn end_pointer_drag(&mut self, outcome: DragOutcome, handler: &mut dyn InputHandler) {
        assert!(
            self.pointer_drag().is_some(),
            "the pointer carries a drag from a source of the tree"
        );
        let last_seen = PointerInput {
            kind: PointerInputKind::Exited,
            position: self.input.pointer.unwrap_or_default(),
            modifiers: self.input.pointer_modifiers,
        };
        self.input.pointer = None;
        let mut pointer = PointerRouter {
            targets: self.pointer_targets(),
            handler,
        };
        DragRouter { pointer: &mut pointer }.end_without_drop(outcome, None, &last_seen);
    }

    /// Cancels the drag the pointer carries, where the pointer is: the target it is over hears
    /// that it left, the source that it was cancelled, and the pointer hovers what it is over.
    pub(super) fn cancel_pointer_drag(&mut self, handler: &mut dyn InputHandler) {
        let (hit, kind, position) = match self.input.pointer {
            Some(position) => (self.element_at(position), PointerInputKind::Moved, position),
            None => (None, PointerInputKind::Exited, Point::default()),
        };
        let input = PointerInput {
            kind,
            position,
            modifiers: self.input.pointer_modifiers,
        };
        let mut pointer = PointerRouter {
            targets: self.pointer_targets(),
            handler,
        };
        DragRouter { pointer: &mut pointer }.end_without_drop(DragOutcome::Cancelled, hit, &input);
    }

    fn follows_system_drag(&self) -> bool {
        self.input
            .drag
            .as_ref()
            .is_some_and(|drag| drag.reach == DragReach::System)
    }

    /// Gives up the drag under way, whoever carries it, where the pointer was last seen.
    fn system_drag_exited_any(&mut self, handler: &mut dyn InputHandler) {
        if self.input.drag.is_none() {
            return;
        }
        let last_seen = PointerInput {
            kind: PointerInputKind::Exited,
            position: self.input.pointer.unwrap_or_default(),
            modifiers: self.input.pointer_modifiers,
        };
        let mut pointer = PointerRouter {
            targets: self.pointer_targets(),
            handler,
        };
        DragRouter { pointer: &mut pointer }.give_up(&last_seen);
        self.input.pointer = None;
    }

    /// Notes where a drag of the system is, and returns the element it is over there with the
    /// place as the routers take it.
    fn follow_system_drag(&mut self, position: Point, modifiers: Modifiers) -> (Option<ElementId>, PointerInput) {
        let hit = self.element_at(position);
        self.input.pointer = Some(position);
        self.input.pointer_modifiers = modifiers;
        self.input.hover_is_stale = false;
        let moved = PointerInput {
            kind: PointerInputKind::Moved,
            position,
            modifiers,
        };
        (hit, moved)
    }
}
