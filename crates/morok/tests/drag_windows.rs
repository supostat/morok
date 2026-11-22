mod support;

use std::cell::Cell;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use morok::components::styles::CLASSES;
use morok::components::view::ElementEvent;
use morok::core::{
    CustomData, DragEventKind, DragOutcome, DragReach, DragStart, ElementId, Payload, PayloadType, Point, Propagation,
    Size,
};
use morok::reactive::Signal;
use morok::{Application, Styles, Window, WindowOptions};
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{NSApplication, NSDragOperation, NSPasteboard, NSPasteboardTypeFileURL, NSWindow};
use objc2_foundation::{NSPoint, NSString};
use support::{EventSource, Expectations};

const CONTENT_SIZE: Size = Size {
    width: 400.0,
    height: 300.0,
};
/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// Long enough for the system to end a drag whose release waits in the queue.
const DRAG_ENDING: Duration = Duration::from_millis(2500);
/// The file a drag of another application carries, and its address on a pasteboard.
const DROPPED_FILE: &str = "/tmp/report.pdf";
const DROPPED_FILE_ADDRESS: &str = "file:///tmp/report.pdf";

/// What a drag of the system tells its destination, set by the test.
struct DragState {
    pasteboard: Retained<NSPasteboard>,
    /// Where the drag is, in the coordinates of the window.
    location: Cell<NSPoint>,
}

define_class!(
    /// A drag of another application as a destination sees it, with what the view of a surface
    /// asks of one. The test hands it to the view in place of the object `AppKit` makes for a
    /// drag.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = DragState]
    struct StagedDrag;

    impl StagedDrag {
        #[unsafe(method(draggingSourceOperationMask))]
        fn dragging_source_operation_mask(&self) -> NSDragOperation {
            NSDragOperation::Copy
        }

        #[unsafe(method(draggingLocation))]
        fn dragging_location(&self) -> NSPoint {
            self.ivars().location.get()
        }

        #[unsafe(method_id(draggingPasteboard))]
        fn dragging_pasteboard(&self) -> Retained<NSPasteboard> {
            self.ivars().pasteboard.clone()
        }

        #[unsafe(method_id(draggingSource))]
        fn dragging_source(&self) -> Option<Retained<AnyObject>> {
            None
        }
    }
);

impl StagedDrag {
    /// A drag that carries what `pasteboard` holds, over the point `at` of the surface.
    fn over(main_thread: MainThreadMarker, pasteboard: Retained<NSPasteboard>, at: Point) -> Retained<Self> {
        let location = NSPoint::new(f64::from(at.x), f64::from(CONTENT_SIZE.height - at.y));
        let drag = Self::alloc(main_thread).set_ivars(DragState {
            pasteboard,
            location: Cell::new(location),
        });
        // SAFETY: the superclass is initialized without arguments.
        unsafe { msg_send![super(drag), init] }
    }
}

/// What the root component of the window built, and the signals its drags write.
#[derive(Clone, Copy, Debug)]
struct Built {
    /// A box the pointer drags out of the window.
    card: ElementId,
    /// A box the pointer drags across the windows of the application.
    row: ElementId,
    /// A box that takes files.
    shelf: ElementId,
    shelf_is_under_drag: Signal<bool>,
    taken_by_shelf: Signal<Payload>,
    card_drag_ended: Signal<Option<DragOutcome>>,
    row_drag_ended: Signal<Option<DragOutcome>>,
}

/// The type of the application's own a row is dragged in.
const ROW_TYPE: &str = "com.example.board.row";

/// What the row carries.
fn row_payload() -> Payload {
    Payload {
        custom: vec![CustomData {
            type_name: String::from(ROW_TYPE),
            bytes: vec![3, 1, 4],
        }],
        ..Payload::default()
    }
}

/// A point far enough from `point` for a press there to become a drag.
fn beside(point: Point) -> Point {
    Point {
        x: point.x + 12.0,
        ..point
    }
}

/// The point of the surface of `window` that lies on the screen where `at` of the surface of
/// `other` lies; both surfaces take `CONTENT_SIZE`.
fn same_place(window: &NSWindow, other: &NSWindow, at: Point) -> Point {
    let in_other = NSPoint::new(f64::from(at.x), f64::from(CONTENT_SIZE.height - at.y));
    let in_window = window.convertPointFromScreen(other.convertPointToScreen(in_other));
    Point {
        x: in_window.x as f32,
        y: CONTENT_SIZE.height - in_window.y as f32,
    }
}

/// The window of the test with what the expectations read of it.
struct Stage {
    main_thread: MainThreadMarker,
    application: Application,
    window: Window,
    source: EventSource,
    pasteboard: Retained<NSPasteboard>,
    expectations: Expectations,
    built: Built,
}

/// What the card carries: data no other application takes, so that the drag the test begins
/// in the system does nothing wherever it ends.
fn card_payload() -> Payload {
    Payload {
        custom: vec![CustomData {
            type_name: String::from("com.example.board.card"),
            bytes: vec![7, 0, 42],
        }],
        ..Payload::default()
    }
}

impl Stage {
    /// A window on a card that hands its drag to the system and a shelf that takes files.
    fn open(main_thread: MainThreadMarker) -> Self {
        let application = Application::new(Styles::of_kit()).expect("this Mac runs Metal 4");
        let (shelf_is_under_drag, taken_by_shelf, card_drag_ended, row_drag_ended) = application.update(|runtime| {
            (
                runtime.signal(false),
                runtime.signal(Payload::default()),
                runtime.signal(None),
                runtime.signal(None),
            )
        });
        let options = WindowOptions {
            title: String::from("morok: drag windows"),
            content_size: CONTENT_SIZE,
            blur_behind: false,
        };
        let mut built = None;
        let window = application.open_window(&options, |mount| {
            let card = mount
                .element()
                .class(CLASSES.button)
                .drag_source(|_| {
                    Some(DragStart {
                        payload: card_payload(),
                        reach: DragReach::System,
                    })
                })
                .on_event(move |runtime, event| {
                    if let ElementEvent::DragEnded(outcome) = event {
                        runtime.set(card_drag_ended, Some(*outcome));
                    }
                    Propagation::Continue
                })
                .id();
            let shelf = mount
                .element()
                .class(CLASSES.button)
                .drop_target(&[PayloadType::Files])
                .on_event(move |runtime, event| {
                    if let ElementEvent::Drag(drag) = event {
                        match drag.kind {
                            DragEventKind::Entered => runtime.set(shelf_is_under_drag, true),
                            DragEventKind::Moved => {}
                            DragEventKind::Left => runtime.set(shelf_is_under_drag, false),
                            DragEventKind::Dropped => {
                                runtime.set(shelf_is_under_drag, false);
                                runtime.set(taken_by_shelf, drag.payload.clone());
                            }
                        }
                    }
                    Propagation::Continue
                })
                .id();
            let row = mount
                .element()
                .class(CLASSES.button)
                .drag_source(|_| {
                    Some(DragStart {
                        payload: row_payload(),
                        reach: DragReach::Surface,
                    })
                })
                .on_event(move |runtime, event| {
                    if let ElementEvent::DragEnded(outcome) = event {
                        runtime.set(row_drag_ended, Some(*outcome));
                    }
                    Propagation::Continue
                })
                .id();
            built = Some(Built {
                card,
                row,
                shelf,
                shelf_is_under_drag,
                taken_by_shelf,
                card_drag_ended,
                row_drag_ended,
            });
            mount.element().child(card).child(shelf).child(row).id()
        });
        let pasteboard = NSPasteboard::pasteboardWithUniqueName();
        pasteboard.clearContents();
        // SAFETY: the type is a constant of AppKit.
        pasteboard.setString_forType(&NSString::from_str(DROPPED_FILE_ADDRESS), unsafe {
            NSPasteboardTypeFileURL
        });
        Self {
            main_thread,
            application,
            window,
            source: EventSource::for_the_window(main_thread, CONTENT_SIZE),
            pasteboard,
            expectations: Expectations::default(),
            built: built.expect("the window mounted its root component"),
        }
    }

    /// The middle of the box the last frame gave the element.
    fn middle_of(&self, element: ElementId) -> Point {
        let border_box = self.window.with_tree(|tree| tree.border_box(element));
        Point {
            x: border_box.x + border_box.width / 2.0,
            y: border_box.y + border_box.height / 2.0,
        }
    }

    fn the_window_asks_the_system_for_the_drags_its_components_take(&mut self) {
        self.application.run_for(SETTLING);
        let asked_for: Vec<String> = self
            .source
            .surface_view()
            .registeredDraggedTypes()
            .to_vec()
            .iter()
            .map(ToString::to_string)
            .collect();
        self.expectations.expect(
            asked_for == ["public.file-url"],
            "the window asks the system for the drags of files, which the shelf takes; is the display on?",
            &format!("{asked_for:?}"),
        );
    }

    fn a_file_another_application_drags_is_dropped_on_a_component(&mut self) {
        let view = self.source.surface_view();
        let drag = StagedDrag::over(
            self.main_thread,
            self.pasteboard.clone(),
            self.middle_of(self.built.shelf),
        );
        // SAFETY: the view implements the method of a dragging destination, which takes the
        // drag and answers with an operation.
        let over_shelf: NSDragOperation = unsafe { msg_send![&*view, draggingEntered: &*drag] };
        let is_under_drag = self
            .application
            .update(|runtime| runtime.get(self.built.shelf_is_under_drag));
        self.expectations.expect(
            over_shelf == NSDragOperation::Copy && is_under_drag,
            "a drag of a file over the shelf is answered with a copy, and the shelf hears that it came",
            &format!("{over_shelf:?}, under the drag: {is_under_drag}"),
        );

        // SAFETY: the view implements the method of a dragging destination, which takes the
        // drag and answers whether it took the drop.
        let was_taken: bool = unsafe { msg_send![&*view, performDragOperation: &*drag] };
        let taken = self
            .application
            .update(|runtime| runtime.get(self.built.taken_by_shelf));
        let expected = Payload {
            files: vec![PathBuf::from(DROPPED_FILE)],
            ..Payload::default()
        };
        self.expectations.expect(
            was_taken && taken == expected,
            "the drop gives the shelf the file the pasteboard of the drag holds",
            &format!("taken: {was_taken}, {taken:?}"),
        );
    }

    fn a_component_hands_its_drag_to_the_system_and_hears_its_end(&mut self) {
        let on_card = self.middle_of(self.built.card);
        let dragged_to = Point {
            x: on_card.x + 12.0,
            y: on_card.y,
        };
        self.source.drag_and_release(self.main_thread, on_card, dragged_to);
        let is_pressed = self
            .window
            .with_tree(|tree| tree.states(self.built.card).contains(morok::style::States::ACTIVE));
        self.expectations.expect(
            !is_pressed,
            "the press on the card ended where the system took its drag",
            &format!("pressed: {is_pressed}"),
        );

        self.application.run_for(DRAG_ENDING);
        let ended = self
            .application
            .update(|runtime| runtime.get(self.built.card_drag_ended));
        self.expectations.expect(
            ended == Some(DragOutcome::Cancelled),
            "the card hears that the drag the system carried ended with no drop",
            &format!("{ended:?}"),
        );
    }
}

impl Stage {
    /// A second window beside the first, with a rack that takes rows.
    fn open_second_window(&self) -> (Window, Retained<NSWindow>, Rack) {
        let (is_under_drag, taken) = self
            .application
            .update(|runtime| (runtime.signal(false), runtime.signal(Payload::default())));
        let options = WindowOptions {
            title: String::from("morok: drag windows, the second"),
            content_size: CONTENT_SIZE,
            blur_behind: false,
        };
        let mut element = None;
        let second = self.application.open_window(&options, |mount| {
            let rack = mount
                .element()
                .class(CLASSES.button)
                .drop_target(&[PayloadType::Custom(String::from(ROW_TYPE))])
                .on_event(move |runtime, event| {
                    if let ElementEvent::Drag(drag) = event {
                        match drag.kind {
                            DragEventKind::Entered => runtime.set(is_under_drag, true),
                            DragEventKind::Moved => {}
                            DragEventKind::Left => runtime.set(is_under_drag, false),
                            DragEventKind::Dropped => {
                                runtime.set(is_under_drag, false);
                                runtime.set(taken, drag.payload.clone());
                            }
                        }
                    }
                    Propagation::Continue
                })
                .id();
            element = Some(rack);
            mount.element().child(rack).id()
        });
        let first = self.source.window();
        let second_native = NSApplication::sharedApplication(self.main_thread)
            .windows()
            .to_vec()
            .into_iter()
            .find(|window| window.windowNumber() != first.windowNumber() && window.isVisible())
            .expect("the application has opened its second window");
        let centered = first.frame();
        let gap = 24.0;
        let left = centered.origin.x - centered.size.width.midpoint(gap);
        first.setFrameOrigin(NSPoint::new(left, centered.origin.y));
        second_native.setFrameOrigin(NSPoint::new(left + centered.size.width + gap, centered.origin.y));
        let rack = Rack {
            element: element.expect("the second window mounted its rack"),
            is_under_drag,
            taken,
        };
        (second, second_native, rack)
    }

    fn a_drag_goes_from_one_window_to_a_target_in_another(&mut self) {
        let (second, second_native, rack) = self.open_second_window();
        self.application.run_for(SETTLING);
        let over_rack = second.with_tree(|tree| {
            let rack_box = tree.border_box(rack.element);
            Point {
                x: rack_box.x + rack_box.width / 2.0,
                y: rack_box.y + rack_box.height / 2.0,
            }
        });
        let over_rack_from_first = same_place(self.source.window(), &second_native, over_rack);
        let on_row = self.middle_of(self.built.row);

        self.source.press(on_row);
        self.source.drag_to(beside(on_row));
        self.source.drag_to(over_rack_from_first);
        let is_under_drag = self.application.update(|runtime| runtime.get(rack.is_under_drag));
        self.expectations.expect(
            is_under_drag,
            "the rack of the second window hears the drag of the row come over it",
            &format!("under the drag: {is_under_drag}, the pointer at {over_rack_from_first:?} of the first window"),
        );

        self.source.release(over_rack_from_first);
        let (taken, ended) = self
            .application
            .update(|runtime| (runtime.get(rack.taken), runtime.get(self.built.row_drag_ended)));
        self.expectations.expect(
            taken == row_payload() && ended == Some(DragOutcome::Dropped),
            "the row is dropped on the rack of the second window, and hears that it was",
            &format!("taken: {taken:?}, ended: {ended:?}"),
        );
        drop(second);
    }
}

/// The drop target of the second window, and the signals its drags write.
#[derive(Clone, Copy, Debug)]
struct Rack {
    element: ElementId,
    is_under_drag: Signal<bool>,
    taken: Signal<Payload>,
}

fn main() -> ExitCode {
    let main_thread = MainThreadMarker::new().expect("the test runs on the main thread");
    let mut stage = Stage::open(main_thread);

    stage.the_window_asks_the_system_for_the_drags_its_components_take();
    stage.a_file_another_application_drags_is_dropped_on_a_component();
    stage.a_component_hands_its_drag_to_the_system_and_hears_its_end();
    stage.a_drag_goes_from_one_window_to_a_target_in_another();

    // SAFETY: a pasteboard with a unique name is released once nothing reads it any more.
    let (): () = unsafe { msg_send![&*stage.pasteboard, releaseGlobally] };
    stage.expectations.report("drag windows")
}
