use morok_style::States;

use crate::drag::drag_event::DragEvent;
use crate::drag::drag_event_kind::DragEventKind;
use crate::drag::drag_outcome::DragOutcome;
use crate::drag::drag_reach::DragReach;
use crate::drag::drag_session::DragSession;
use crate::element_id::ElementId;
use crate::input::pointer_button::PointerButton;
use crate::input::pointer_input::PointerInput;
use crate::input::pointer_input_kind::PointerInputKind;
use crate::input::pointer_router::PointerRouter;
use crate::paint::group_transform::surface_transform;

/// Carries a drag across the drop targets of a tree: keeps the target the drag is over
/// hovered, with its ancestors and nothing else, and tells the targets and the source what
/// the drag does.
pub(crate) struct DragRouter<'r, 'a> {
    pub(crate) pointer: &'r mut PointerRouter<'a>,
}

impl DragRouter<'_, '_> {
    /// Starts to follow a drag. The pointer hovers nothing from here on: the elements it was
    /// over hear that it left.
    pub(crate) fn begin(&mut self, session: DragSession, input: &PointerInput) {
        self.pointer.hover(None, input);
        self.pointer.targets.input.drag = Some(session);
    }

    /// Takes what the pointer did while it carries the drag: the drag follows its moves and
    /// is dropped where the primary button comes up. The other buttons do nothing to it.
    pub(crate) fn pointer_input(&mut self, input: &PointerInput, hit: Option<ElementId>) {
        match input.kind {
            PointerInputKind::Moved => {
                self.move_to(hit, input);
            }
            PointerInputKind::Exited => self.retarget(None, input),
            PointerInputKind::Released {
                button: PointerButton::Primary,
                ..
            } => {
                self.drop_at(hit, input);
            }
            PointerInputKind::Pressed { .. } | PointerInputKind::Released { .. } => {}
        }
    }

    /// The drag moved over `hit`. Returns whether a drop target takes it there.
    pub(crate) fn move_to(&mut self, hit: Option<ElementId>, input: &PointerInput) -> bool {
        self.retarget(hit, input);
        let target = self.target();
        if let Some(target) = target {
            self.tell(target, input, DragEventKind::Moved);
        }
        target.is_some()
    }

    /// Works out the target of a drag over `hit` and tells the target the drag left and the
    /// target it came over.
    pub(crate) fn retarget(&mut self, hit: Option<ElementId>, input: &PointerInput) {
        let before = self.target();
        let target = self.target_at(hit);
        if target == before {
            return;
        }
        if let Some(left) = before {
            self.tell(left, input, DragEventKind::Left);
        }
        self.pointer.targets.set_hovered(target);
        if let Some(entered) = target {
            self.tell(entered, input, DragEventKind::Entered);
        }
    }

    /// Ends the drag with a drop over `hit`. Returns whether a drop target took it.
    pub(crate) fn drop_at(&mut self, hit: Option<ElementId>, input: &PointerInput) -> bool {
        self.retarget(hit, input);
        let target = self.target();
        if let Some(target) = target {
            self.tell(target, input, DragEventKind::Dropped);
        }
        let outcome = if target.is_some() {
            DragOutcome::Dropped
        } else {
            DragOutcome::Cancelled
        };
        self.end(outcome, hit, input);
        target.is_some()
    }

    /// Ends the drag where nothing takes it: it left the surface for good.
    pub(crate) fn give_up(&mut self, input: &PointerInput) {
        if let Some(target) = self.target() {
            self.tell(target, input, DragEventKind::Left);
        }
        self.end(DragOutcome::Cancelled, None, input);
    }

    /// The drag is over. The source of a drag the pointer carried hears how it ended, and the
    /// pointer hovers what it is over again; where the pointer is after a drag of the system
    /// is told by its next input.
    fn end(&mut self, outcome: DragOutcome, hit: Option<ElementId>, input: &PointerInput) {
        self.pointer.targets.set_hovered(None);
        let session = self
            .pointer
            .targets
            .input
            .drag
            .take()
            .expect("a drag is under way while it is routed");
        if session.reach == DragReach::System {
            return;
        }
        if let Some(source) = session.source {
            self.pointer.handler.drag_ended(source, outcome);
        }
        self.pointer.hover(hit, input);
    }

    /// The drop target the drag is over.
    fn target(&self) -> Option<ElementId> {
        self.pointer.targets.input.hovered
    }

    /// The drop target of a drag over `hit`: the innermost element at or around `hit` that
    /// takes a type the drag carries and is not disabled.
    fn target_at(&self, hit: Option<ElementId>) -> Option<ElementId> {
        let targets = &self.pointer.targets;
        let payload = &targets.input.drag.as_ref()?.payload;
        let elements = &*targets.marker.elements;
        let mut current = hit;
        while let Some(element) = current {
            let held = &elements[element];
            let takes_drag = held.drop_types().iter().any(|taken| payload.holds(taken));
            if takes_drag && !held.states.contains(States::DISABLED) {
                return Some(element);
            }
            current = held.parent;
        }
        None
    }

    fn tell(&mut self, target: ElementId, input: &PointerInput, kind: DragEventKind) {
        let pointer = &mut *self.pointer;
        let elements = &*pointer.targets.marker.elements;
        let session = pointer
            .targets
            .input
            .drag
            .as_ref()
            .expect("a drag is under way while it is routed");
        let event = DragEvent {
            kind,
            position: input.position,
            modifiers: input.modifiers,
            payload: &session.payload,
            element_box: surface_transform(elements, target).bounds_of(&elements[target].border_box),
        };
        pointer.handler.drag_event(target, &event);
    }
}
