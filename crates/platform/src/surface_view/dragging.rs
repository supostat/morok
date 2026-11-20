use morok_core::{DragOutcome, Point};
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{AnyThread, DefinedClass};
use objc2_app_kit::{NSDragOperation, NSDraggingContext, NSDraggingInfo, NSDraggingItem, NSEvent, NSPasteboardWriting};
use objc2_foundation::{NSArray, NSPoint, NSRect, NSSize};

use super::SurfaceView;
use crate::drag_picture::DragPicture;
use crate::input_translation;
use crate::pasteboard_payload;

/// The drags of the system: the drag the content hands to the system, which the view begins
/// there with the picture of its source, and the drag the system carries over the view, which
/// the view carries over the content.
impl SurfaceView {
    /// Begins in the system the drag the content handed to it when the pointer moved with the
    /// primary button down; `event` is that move. A drag that carries nothing is over at once.
    pub(super) fn begin_requested_drag(&self, event: &NSEvent) {
        let Some(request) = self.content().borrow_mut().system_drag() else {
            return;
        };
        let carried = pasteboard_payload::items_of(&request.payload);
        if carried.is_empty() {
            self.content().borrow_mut().system_drag_ended(DragOutcome::Cancelled);
            return;
        }
        let picture = request.picture_box.and_then(|covered| {
            DragPicture::render(
                self.renderer(),
                &request.picture,
                &request.theme,
                &covered,
                &self.target_geometry(),
            )
        });
        let frame = picture
            .as_ref()
            .map_or_else(|| self.point_under(event), |picture| ns_rect(&picture.frame));
        let dragged: Vec<Retained<NSDraggingItem>> = carried
            .iter()
            .enumerate()
            .map(|(position, item)| {
                let writer: &ProtocolObject<dyn NSPasteboardWriting> = ProtocolObject::from_ref(&**item);
                let dragged = NSDraggingItem::initWithPasteboardWriter(NSDraggingItem::alloc(), writer);
                // The picture shows the whole source, so the first item carries it alone.
                let contents = picture
                    .as_ref()
                    .filter(|_| position == 0)
                    .map(|picture| -> &AnyObject { &picture.image });
                // SAFETY: the contents of a dragging item are an image.
                unsafe { dragged.setDraggingFrame_contents(frame, contents) };
                dragged
            })
            .collect();
        self.beginDraggingSessionWithItems_event_source(
            &NSArray::from_retained_slice(&dragged),
            event,
            ProtocolObject::from_ref(self),
        );
    }

    /// What a drag the view began may do where it is dropped: inside the application the
    /// drop target decides, and another application is given a copy.
    pub(super) fn offered_operations(context: NSDraggingContext) -> NSDragOperation {
        if context == NSDraggingContext::WithinApplication {
            NSDragOperation::Generic
        } else {
            NSDragOperation::Copy
        }
    }

    /// The drag the view began in the system is over: `operation` is what its destination did
    /// with it, nothing when none took it.
    pub(super) fn handed_drag_ended(&self, operation: NSDragOperation) {
        let outcome = if operation == NSDragOperation::None {
            DragOutcome::Cancelled
        } else {
            DragOutcome::Dropped
        };
        self.content().borrow_mut().system_drag_ended(outcome);
    }

    /// Asks the system to offer the view the drags its content takes, when the payload types
    /// the content takes changed since it was asked last.
    pub(super) fn sync_drop_types(&self) {
        let state = self.ivars();
        let revision = self.content().borrow_mut().drop_types_revision();
        if state.registered_drop_types.get() == Some(revision) {
            return;
        }
        let types = self.content().borrow_mut().drop_types();
        if types.is_empty() {
            self.unregisterDraggedTypes();
        } else {
            self.registerForDraggedTypes(&pasteboard_payload::pasteboard_types(&types));
        }
        state.drop_types.replace(types);
        state.registered_drop_types.set(Some(revision));
    }

    /// A drag of the system came over the view: the content is given what the drag carries in
    /// the types it takes.
    pub(super) fn drag_came_over(&self, drag: &ProtocolObject<dyn NSDraggingInfo>) -> NSDragOperation {
        let payload = pasteboard_payload::read(&drag.draggingPasteboard(), &self.ivars().drop_types.borrow());
        let is_taken = self.content().borrow_mut().system_drag_entered(
            payload,
            self.place_of(drag),
            input_translation::modifiers(NSEvent::modifierFlags_class()),
        );
        Self::answer_to(drag, is_taken)
    }

    pub(super) fn drag_moved_over(&self, drag: &ProtocolObject<dyn NSDraggingInfo>) -> NSDragOperation {
        let is_taken = self.content().borrow_mut().system_drag_moved(
            self.place_of(drag),
            input_translation::modifiers(NSEvent::modifierFlags_class()),
        );
        Self::answer_to(drag, is_taken)
    }

    /// The drag left the view, or ended somewhere else.
    pub(super) fn drag_left(&self) {
        self.content().borrow_mut().system_drag_exited();
    }

    /// Returns whether the content took the drop.
    pub(super) fn drag_dropped(&self, drag: &ProtocolObject<dyn NSDraggingInfo>) -> bool {
        self.content().borrow_mut().system_drag_dropped(
            self.place_of(drag),
            input_translation::modifiers(NSEvent::modifierFlags_class()),
        )
    }

    /// What the view answers the system about a drag over it: nothing while no drop target
    /// takes the drag, else what a drag of the application itself was begun for, and a copy
    /// of what another application drags.
    fn answer_to(drag: &ProtocolObject<dyn NSDraggingInfo>, is_taken: bool) -> NSDragOperation {
        let wanted = if drag.draggingSource().is_some() {
            NSDragOperation::Generic
        } else {
            NSDragOperation::Copy
        };
        if is_taken && drag.draggingSourceOperationMask().contains(wanted) {
            wanted
        } else {
            NSDragOperation::None
        }
    }

    /// Where the drag is on the surface, in CSS pixels.
    fn place_of(&self, drag: &ProtocolObject<dyn NSDraggingInfo>) -> Point {
        let in_view = self.convertPoint_fromView(drag.draggingLocation(), None);
        Point {
            x: in_view.x as f32,
            y: in_view.y as f32,
        }
    }

    /// The point of the view under the pointer of a mouse event, as the frame of a drag that
    /// shows no picture.
    fn point_under(&self, event: &NSEvent) -> NSRect {
        NSRect::new(
            self.convertPoint_fromView(event.locationInWindow(), None),
            NSSize::new(1.0, 1.0),
        )
    }
}

fn ns_rect(rect: &morok_core::Rect) -> NSRect {
    NSRect::new(
        NSPoint::new(f64::from(rect.x), f64::from(rect.y)),
        NSSize::new(f64::from(rect.width), f64::from(rect.height)),
    )
}
