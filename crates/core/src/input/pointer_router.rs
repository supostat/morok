use crate::element_id::ElementId;
use crate::input::input_handler::InputHandler;
use crate::input::pointer_button::PointerButton;
use crate::input::pointer_event::PointerEvent;
use crate::input::pointer_event_kind::PointerEventKind;
use crate::input::pointer_input::PointerInput;
use crate::input::pointer_input_kind::PointerInputKind;
use crate::input::pointer_targets::PointerTargets;
use crate::input::propagation::Propagation;

/// Routes what the pointer does to the elements it concerns.
///
/// While the primary button is down the pointer belongs to the element it went down on: that
/// element hears of the moves and of the release wherever they happen, and only it and its
/// ancestors can be hovered.
pub(crate) struct PointerRouter<'a> {
    pub(crate) targets: PointerTargets<'a>,
    pub(crate) handler: &'a mut dyn InputHandler,
}

impl PointerRouter<'_> {
    /// Routes `input`, made over the element `hit`.
    pub(crate) fn route(&mut self, input: &PointerInput, hit: Option<ElementId>) {
        self.hover(hit, input);
        match input.kind {
            PointerInputKind::Exited => {}
            PointerInputKind::Moved => self.bubble(self.receiver(hit), input, PointerEventKind::Moved),
            PointerInputKind::Pressed { button, click_count } => {
                if button == PointerButton::Primary {
                    self.targets.set_pressed(hit);
                }
                let pressed = PointerEventKind::Pressed { button, click_count };
                self.bubble(self.receiver(hit), input, pressed);
            }
            PointerInputKind::Released { button, click_count } => {
                self.bubble(self.receiver(hit), input, PointerEventKind::Released { button });
                if button == PointerButton::Primary {
                    self.end_press(hit, input, click_count);
                }
            }
        }
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
        let event = PointerEvent {
            kind,
            position: input.position,
            modifiers: input.modifiers,
        };
        self.handler.pointer_event(element, &event)
    }

    /// Offers the event to `innermost` and then to each element around it, until one uses it.
    fn bubble(&mut self, innermost: Option<ElementId>, input: &PointerInput, kind: PointerEventKind) {
        let mut receiver = innermost;
        while let Some(element) = receiver {
            if self.tell(element, input, kind) == Propagation::Stop {
                return;
            }
            receiver = self.targets.marker.elements[element].parent;
        }
    }
}
