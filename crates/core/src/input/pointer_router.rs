use morok_style::States;

use crate::drag::drag_candidate::DragCandidate;
use crate::drag::drag_reach::DragReach;
use crate::drag::drag_router::DragRouter;
use crate::drag::drag_session::DragSession;
use crate::drag::system_drag::SystemDrag;
use crate::element_id::ElementId;
use crate::input::input_handler::InputHandler;
use crate::input::pointer_button::PointerButton;
use crate::input::pointer_event::PointerEvent;
use crate::input::pointer_event_kind::PointerEventKind;
use crate::input::pointer_input::PointerInput;
use crate::input::pointer_input_kind::PointerInputKind;
use crate::input::pointer_targets::PointerTargets;
use crate::input::propagation::Propagation;
use crate::paint::group_transform::surface_transform;

/// Routes what the pointer does to the elements it concerns.
///
/// While the primary button is down the pointer belongs to the element it went down on: that
/// element hears of the moves and of the release wherever they happen, and only it and its
/// ancestors can be hovered.
///
/// A press on a drag source becomes a drag once the pointer has travelled far enough: the
/// press ends without a click, and the pointer carries the drag until the button comes up.
pub(crate) struct PointerRouter<'a> {
    pub(crate) targets: PointerTargets<'a>,
    pub(crate) handler: &'a mut dyn InputHandler,
}

impl PointerRouter<'_> {
    /// Routes `input`, made over the element `hit`.
    pub(crate) fn route(&mut self, input: &PointerInput, hit: Option<ElementId>) {
        match self.targets.input.drag.as_ref().map(|drag| drag.reach) {
            Some(DragReach::Surface) => {
                DragRouter { pointer: self }.pointer_input(input, hit);
                return;
            }
            // The system delivers no pointer input while it carries a drag: the drag is over.
            Some(DragReach::System) => DragRouter { pointer: self }.give_up(input),
            None => {}
        }
        if input.kind == PointerInputKind::Moved && self.begins_drag(input, hit) {
            return;
        }
        self.hover(hit, input);
        match input.kind {
            PointerInputKind::Exited => {}
            PointerInputKind::Moved => {
                self.bubble(self.receiver(hit), input, PointerEventKind::Moved);
            }
            PointerInputKind::Pressed { button, click_count } => {
                if button == PointerButton::Primary {
                    self.targets.set_pressed(hit);
                }
                let pressed = PointerEventKind::Pressed { button, click_count };
                let used_by = self.bubble(self.receiver(hit), input, pressed);
                if button == PointerButton::Primary {
                    self.targets.input.drag_candidate =
                        self.drag_source_around(hit, used_by).map(|source| DragCandidate {
                            source,
                            pressed_at: input.position,
                        });
                }
            }
            PointerInputKind::Released { button, click_count } => {
                self.bubble(self.receiver(hit), input, PointerEventKind::Released { button });
                if button == PointerButton::Primary {
                    self.targets.input.drag_candidate = None;
                    self.end_press(hit, input, click_count);
                }
            }
        }
    }

    /// Turns the press into a drag once the pointer has travelled far enough from where the
    /// button went down on a drag source, and the source gives the drag something to carry.
    /// Returns whether a drag began. A press that selects in an edited text stays a selection.
    fn begins_drag(&mut self, input: &PointerInput, hit: Option<ElementId>) -> bool {
        let Some(candidate) = self.targets.input.drag_candidate else {
            return false;
        };
        if !candidate.has_travelled_to(input.position) {
            return false;
        }
        self.targets.input.drag_candidate = None;
        if self.targets.input.text_drag.is_some() {
            return false;
        }
        let Some(start) = self.handler.drag_started(candidate.source) else {
            return false;
        };
        self.targets.set_pressed(None);
        match start.reach {
            DragReach::Surface => {
                let mut drag = DragRouter { pointer: self };
                drag.begin(
                    DragSession {
                        payload: start.payload,
                        reach: DragReach::Surface,
                        source: Some(candidate.source),
                    },
                    input,
                );
                drag.move_to(hit, input);
            }
            DragReach::System => {
                self.targets.input.system_drag = Some(SystemDrag {
                    source: candidate.source,
                    payload: start.payload,
                });
                self.hover(hit, input);
            }
        }
        true
    }

    /// The drag source a press over `hit` may drag: the innermost one at or around `hit` that
    /// is not disabled, no further out than the element that used the press.
    fn drag_source_around(&self, hit: Option<ElementId>, used_by: Option<ElementId>) -> Option<ElementId> {
        let elements = &*self.targets.marker.elements;
        let mut current = hit;
        while let Some(element) = current {
            let held = &elements[element];
            if held.is_drag_source() && !held.states.contains(States::DISABLED) {
                return Some(element);
            }
            if used_by == Some(element) {
                return None;
            }
            current = held.parent;
        }
        None
    }

    /// Tells the elements the pointer left and the elements it came over, and hovers the
    /// element it is over now.
    pub(crate) fn hover(&mut self, hit: Option<ElementId>, input: &PointerInput) {
        let elements = &*self.targets.marker.elements;
        let hovered = match self.targets.input.pressed {
            Some(pressed) => elements.common_ancestor(Some(pressed), hit),
            None => hit,
        };
        let before = self.targets.input.hovered;
        if hovered == before {
            return;
        }
        let kept = elements.common_ancestor(before, hovered);
        let mut left = before;
        while let Some(element) = left
            && left != kept
        {
            self.tell(element, input, PointerEventKind::Left);
            left = self.targets.marker.elements[element].parent;
        }
        self.targets.set_hovered(hovered);
        self.targets.input.entered.clear();
        let mut entered = hovered;
        while let Some(element) = entered
            && entered != kept
        {
            self.targets.input.entered.push(element);
            entered = self.targets.marker.elements[element].parent;
        }
        for position in (0..self.targets.input.entered.len()).rev() {
            let element = self.targets.input.entered[position];
            self.tell(element, input, PointerEventKind::Entered);
        }
    }

    /// The element that hears of what the pointer does over `hit`.
    fn receiver(&self, hit: Option<ElementId>) -> Option<ElementId> {
        self.targets.input.pressed.or(hit)
    }

    fn end_press(&mut self, hit: Option<ElementId>, input: &PointerInput, click_count: u32) {
        let Some(pressed) = self.targets.input.pressed else {
            return;
        };
        let clicked = self.targets.marker.elements.common_ancestor(Some(pressed), hit);
        self.bubble(clicked, input, PointerEventKind::Clicked { click_count });
        self.targets.set_pressed(None);
        self.hover(hit, input);
    }

    fn tell(&mut self, element: ElementId, input: &PointerInput, kind: PointerEventKind) -> Propagation {
        let elements = &*self.targets.marker.elements;
        let event = PointerEvent {
            kind,
            position: input.position,
            modifiers: input.modifiers,
            element_box: surface_transform(elements, element).bounds_of(&elements[element].border_box),
        };
        self.handler.pointer_event(element, &event)
    }

    /// Offers the event to `innermost` and then to each element around it, until one uses it.
    /// Returns the element that used it.
    fn bubble(
        &mut self,
        innermost: Option<ElementId>,
        input: &PointerInput,
        kind: PointerEventKind,
    ) -> Option<ElementId> {
        let mut receiver = innermost;
        while let Some(element) = receiver {
            if self.tell(element, input, kind) == Propagation::Stop {
                return Some(element);
            }
            receiver = self.targets.marker.elements[element].parent;
        }
        None
    }
}
