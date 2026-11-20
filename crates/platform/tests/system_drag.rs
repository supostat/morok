mod support;

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use morok_core::accesskit::{ActionRequest, TreeUpdate};
use morok_core::{
    AccessibilityAction, Clipboard, CustomData, DisplayList, DragEvent, DragEventKind, DragOutcome, DragReach,
    DragStart, ElementId, ElementTree, InputHandler, KeyEvent, Modifiers, Payload, PayloadType, Point, PointerEvent,
    PointerInput, Propagation, Rect, ScrollInput, ShortcutId, Size,
};
use morok_platform::{Application, PasteboardClipboard, SurfaceContent, SystemDragRequest, Window, WindowOptions};
use morok_style::{ColorScheme, States};
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSApplication, NSDragOperation, NSEvent, NSEventModifierFlags, NSEventType, NSPasteboard, NSView, NSWindow,
};
use objc2_foundation::NSPoint;
use support::{Expectations, WindowScene, run_main_loop};

const BOARD: &str = ".board { position: relative; width: 320px; height: 200px; background-color: var(--color-paper); }\n\
    .card { position: absolute; left: 20px; top: 20px; width: 80px; height: 40px; \
            background-color: var(--color-red); }\n\
    .bin { position: absolute; left: 180px; top: 20px; width: 120px; height: 160px; \
           background-color: var(--color-green); }\n\
    .bin:hover { background-color: var(--color-blue); }";

const CONTENT_SIZE: Size = Size {
    width: 320.0,
    height: 200.0,
};
const ON_CARD: Point = Point { x: 60.0, y: 40.0 };
const DRAGGED: Point = Point { x: 75.0, y: 40.0 };
const OVER_BOARD: Point = Point { x: 140.0, y: 190.0 };
const OVER_BIN: Point = Point { x: 240.0, y: 100.0 };

/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// Long enough for the system to end a drag whose pointer holds no button down.
const DRAG_ENDING: Duration = Duration::from_millis(2500);

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

fn dropped_files() -> Payload {
    Payload {
        files: vec![PathBuf::from("/tmp/report.pdf"), PathBuf::from("/tmp/notes")],
        ..Payload::default()
    }
}

/// What the tree told its elements of the drags.
#[derive(Debug, Default)]
struct Heard {
    /// What the card answers the pointer that begins to drag it.
    drag_start: Option<DragStart>,
    dragged_sources: Vec<ElementId>,
    drag_events: Vec<(ElementId, DragEventKind, Payload)>,
    ended_drags: Vec<(ElementId, DragOutcome)>,
}

impl InputHandler for Heard {
    fn pointer_event(&mut self, _element: ElementId, _event: &PointerEvent) -> Propagation {
        Propagation::Continue
    }

    fn key_event(&mut self, _element: ElementId, _event: &KeyEvent) -> Propagation {
        Propagation::Continue
    }

    fn shortcut(&mut self, _shortcut: ShortcutId) {}

    fn accessibility_action(&mut self, _element: ElementId, _action: &AccessibilityAction<'_>) {}

    fn text_changed(&mut self, _element: ElementId, _text: &str) {}

    fn drag_started(&mut self, source: ElementId) -> Option<DragStart> {
        self.dragged_sources.push(source);
        self.drag_start.clone()
    }

    fn drag_event(&mut self, target: ElementId, event: &DragEvent<'_>) {
        self.drag_events.push((target, event.kind, event.payload.clone()));
    }

    fn drag_ended(&mut self, source: ElementId, outcome: DragOutcome) {
        self.ended_drags.push((source, outcome));
    }
}

/// A scene in a window that hands the drags of its tree to the window and carries the drags
/// of the window over its tree, as a host of the kit does.
struct DragScene {
    window_scene: WindowScene,
    heard: Heard,
    /// The drag source of the drag handed to the window and not over yet.
    dragged_source: Option<ElementId>,
    /// The picture of every drag handed to the window: the rectangle it covers and the number
    /// of its display items.
    handed_pictures: Vec<(Option<Rect>, usize)>,
}

impl SurfaceContent for DragScene {
    fn frame(&mut self, viewport: Size, scale: f32, now: Duration, show: &mut dyn FnMut(&ElementTree)) {
        self.window_scene.scene.tree.refresh_hover(&mut self.heard);
        self.window_scene.frame(viewport, scale, now, show);
    }

    fn color_scheme_changed(&mut self, color_scheme: ColorScheme) {
        self.window_scene.color_scheme_changed(color_scheme);
    }

    fn reduced_motion_changed(&mut self, reduces_motion: bool) {
        self.window_scene.reduced_motion_changed(reduces_motion);
    }

    fn pointer_input(&mut self, input: &PointerInput) {
        self.window_scene.scene.tree.pointer_input(input, &mut self.heard);
    }

    fn scroll_input(&mut self, _input: &ScrollInput) {}

    fn key_input(&mut self, _event: &KeyEvent) -> bool {
        false
    }

    fn accessibility_tree(&mut self) -> TreeUpdate {
        self.window_scene.accessibility_tree()
    }

    fn accessibility_update(&mut self) -> TreeUpdate {
        self.window_scene.accessibility_update()
    }

    fn accessibility_action(&mut self, _request: &ActionRequest) {}

    fn system_drag(&mut self) -> Option<SystemDragRequest> {
        let tree = &mut self.window_scene.scene.tree;
        let drag = tree.take_system_drag()?;
        let mut picture = DisplayList::default();
        let picture_box = tree.subtree_display_list(drag.source, &mut picture);
        self.dragged_source = Some(drag.source);
        self.handed_pictures.push((picture_box, picture.items().len()));
        Some(SystemDragRequest {
            payload: drag.payload,
            picture,
            theme: tree.theme().clone(),
            picture_box,
        })
    }

    fn system_drag_ended(&mut self, outcome: DragOutcome) {
        let source = self
            .dragged_source
            .take()
            .expect("the window tells the end of a drag it was handed");
        self.window_scene
            .scene
            .tree
            .system_drag_ended(source, outcome, &mut self.heard);
    }

    fn drop_types_revision(&mut self) -> u64 {
        self.window_scene.scene.tree.drop_types_revision()
    }

    fn drop_types(&mut self) -> Vec<PayloadType> {
        self.window_scene.scene.tree.accepted_drop_types()
    }

    fn system_drag_entered(&mut self, payload: Payload, position: Point, modifiers: Modifiers) -> bool {
        self.window_scene
            .scene
            .tree
            .system_drag_entered(payload, position, modifiers, &mut self.heard)
    }

    fn system_drag_moved(&mut self, position: Point, modifiers: Modifiers) -> bool {
        self.window_scene
            .scene
            .tree
            .system_drag_moved(position, modifiers, &mut self.heard)
    }

    fn system_drag_exited(&mut self) {
        self.window_scene.scene.tree.system_drag_exited(&mut self.heard);
    }

    fn system_drag_dropped(&mut self, position: Point, modifiers: Modifiers) -> bool {
        self.window_scene
            .scene
            .tree
            .system_drag_dropped(position, modifiers, &mut self.heard)
    }
}

/// What a drag of the system tells its destination, set by the test.
struct DragState {
    pasteboard: Retained<NSPasteboard>,
    /// Where the drag is, in the coordinates of the window.
    location: Cell<NSPoint>,
    offered: Cell<NSDragOperation>,
    /// The source of a drag that began in this application.
    source: RefCell<Option<Retained<AnyObject>>>,
}

define_class!(
    /// A drag of the system as a destination sees it, with what the view of a surface asks of
    /// one. The test hands it to the view in place of the object `AppKit` makes for a drag.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = DragState]
    struct StagedDrag;

    impl StagedDrag {
        #[unsafe(method(draggingSourceOperationMask))]
        fn dragging_source_operation_mask(&self) -> NSDragOperation {
            self.ivars().offered.get()
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
            self.ivars().source.borrow().clone()
        }
    }
);

impl StagedDrag {
    /// A drag of another application that carries what `pasteboard` holds and may be copied.
    fn new(main_thread: MainThreadMarker, pasteboard: Retained<NSPasteboard>) -> Retained<Self> {
        let drag = Self::alloc(main_thread).set_ivars(DragState {
            pasteboard,
            location: Cell::new(NSPoint::ZERO),
            offered: Cell::new(NSDragOperation::Copy),
            source: RefCell::new(None),
        });
        // SAFETY: the superclass is initialized without arguments.
        unsafe { msg_send![super(drag), init] }
    }

    fn move_to(&self, at: Point) {
        self.ivars().location.set(location(at));
    }
}

/// A point of the surface in the coordinates of the window, which measures from the bottom.
fn location(at: Point) -> NSPoint {
    NSPoint::new(f64::from(at.x), f64::from(CONTENT_SIZE.height - at.y))
}

/// The window of the test, its view as the system reaches it, and what the test expects.
struct Stage {
    window: Window<DragScene>,
    native_window: Retained<NSWindow>,
    view: Retained<NSView>,
    main_thread: MainThreadMarker,
    pasteboard: Retained<NSPasteboard>,
    expectations: Expectations,
    card: ElementId,
    bin: ElementId,
}

impl Stage {
    /// A window on a board with a card that is a drag source and a bin that takes files.
    fn open(application: &Application, main_thread: MainThreadMarker) -> Self {
        let options = WindowOptions {
            title: String::from("morok: system drag"),
            content_size: CONTENT_SIZE,
            blur_behind: false,
        };
        let mut built = None;
        let window = Window::open(application, &options, |frame_scheduler, settings| {
            let mut window_scene = WindowScene::new(BOARD, frame_scheduler, settings);
            let scene = &mut window_scene.scene;
            let root = scene.root();
            scene.add_classes(root, &["board"]);
            let card = scene.add(root, &["card"]);
            let bin = scene.add(root, &["bin"]);
            scene.tree.set_drag_source(card, true);
            scene.tree.set_drop_types(bin, &[PayloadType::Files]);
            built = Some((card, bin));
            DragScene {
                window_scene,
                heard: Heard::default(),
                dragged_source: None,
                handed_pictures: Vec::new(),
            }
        });
        let (card, bin) = built.expect("the window built its content");
        let native_window = NSApplication::sharedApplication(main_thread)
            .windows()
            .firstObject()
            .expect("the application has opened its window");
        let view = native_window.contentView().expect("the window shows its surface");
        let pasteboard = NSPasteboard::pasteboardWithUniqueName();
        PasteboardClipboard::of(pasteboard.clone()).write(&dropped_files());
        Self {
            window,
            native_window,
            view,
            main_thread,
            pasteboard,
            expectations: Expectations::default(),
            card,
            bin,
        }
    }

    fn drag_events(&self) -> Vec<(ElementId, DragEventKind)> {
        self.window.update(|scene| {
            scene
                .heard
                .drag_events
                .iter()
                .map(|(target, kind, _)| (*target, *kind))
                .collect()
        })
    }

    fn bin_is_hovered(&self) -> bool {
        self.window
            .update(|scene| scene.window_scene.scene.tree.states(self.bin).contains(States::HOVER))
    }

    fn drag_came_over(&self, drag: &StagedDrag) -> NSDragOperation {
        // SAFETY: the view implements the method of a dragging destination, which takes the
        // drag and answers with an operation.
        unsafe { msg_send![&*self.view, draggingEntered: drag] }
    }

    fn drag_moved_over(&self, drag: &StagedDrag) -> NSDragOperation {
        // SAFETY: as for the drag that comes over the view.
        unsafe { msg_send![&*self.view, draggingUpdated: drag] }
    }

    fn drag_left(&self, drag: &StagedDrag) {
        // SAFETY: the view implements the method of a dragging destination, which takes the
        // drag.
        let (): () = unsafe { msg_send![&*self.view, draggingExited: drag] };
    }

    fn drag_dropped(&self, drag: &StagedDrag) -> bool {
        // SAFETY: the view implements the method of a dragging destination, which takes the
        // drag and answers whether it took the drop.
        unsafe { msg_send![&*self.view, performDragOperation: drag] }
    }

    fn the_view_asks_for_the_drags_its_targets_take(&mut self) {
        let asked_for: Vec<String> = self
            .view
            .registeredDraggedTypes()
            .to_vec()
            .iter()
            .map(ToString::to_string)
            .collect();
        self.expectations.expect(
            asked_for == ["public.file-url"],
            "the view asks the system for the drags of files, which the bin takes",
            &format!("{asked_for:?}"),
        );
    }

    fn a_drag_of_another_application_is_carried_over_the_tree_and_dropped(&mut self) {
        let drag = StagedDrag::new(self.main_thread, self.pasteboard.clone());

        drag.move_to(OVER_BOARD);
        let over_board = self.drag_came_over(&drag);
        self.expectations.expect(
            over_board == NSDragOperation::None && self.drag_events().is_empty(),
            "a drag over the board, which takes nothing, is answered with no operation",
            &format!("{over_board:?}, {:?}", self.drag_events()),
        );

        drag.move_to(OVER_BIN);
        let over_bin = self.drag_moved_over(&drag);
        self.expectations.expect(
            over_bin == NSDragOperation::Copy && self.bin_is_hovered(),
            "a drag of files over the bin is answered with a copy, and the bin is hovered",
            &format!("{over_bin:?}, hovered: {}", self.bin_is_hovered()),
        );

        let was_taken = self.drag_dropped(&drag);
        let heard = self.window.update(|scene| scene.heard.drag_events.clone());
        let expected = [DragEventKind::Entered, DragEventKind::Moved, DragEventKind::Dropped]
            .map(|kind| (self.bin, kind, dropped_files()));
        self.expectations.expect(
            was_taken && heard == expected && !self.bin_is_hovered(),
            "the drop gives the bin the files the pasteboard of the drag holds",
            &format!("taken: {was_taken}, {heard:?}"),
        );
    }

    fn a_drag_that_leaves_is_over_for_the_tree(&mut self) {
        let drag = StagedDrag::new(self.main_thread, self.pasteboard.clone());
        drag.move_to(OVER_BIN);
        let over_bin = self.drag_came_over(&drag);
        self.drag_left(&drag);
        let last = self.drag_events().last().copied();
        let follows_a_drag = self
            .window
            .update(|scene| scene.window_scene.scene.tree.dragged_payload().is_some());
        self.expectations.expect(
            over_bin == NSDragOperation::Copy
                && last == Some((self.bin, DragEventKind::Left))
                && !follows_a_drag
                && !self.bin_is_hovered(),
            "a drag that leaves the view leaves the bin, and the tree follows it no longer",
            &format!("{over_bin:?}, {last:?}, follows a drag: {follows_a_drag}"),
        );
        let dropped_after = self.drag_dropped(&drag);
        self.expectations.expect(
            !dropped_after,
            "a drop of a drag that left takes nothing",
            &format!("{dropped_after}"),
        );
    }

    fn the_answer_follows_where_the_drag_began_and_what_its_source_allows(&mut self) {
        let from_inside = StagedDrag::new(self.main_thread, self.pasteboard.clone());
        from_inside
            .ivars()
            .source
            .replace(Some(Retained::into_super(Retained::into_super(Retained::into_super(
                self.view.clone(),
            )))));
        from_inside.ivars().offered.set(NSDragOperation::Generic);
        from_inside.move_to(OVER_BIN);
        let inside = self.drag_came_over(&from_inside);
        self.drag_left(&from_inside);

        let no_copy = StagedDrag::new(self.main_thread, self.pasteboard.clone());
        no_copy.ivars().offered.set(NSDragOperation::Link);
        no_copy.move_to(OVER_BIN);
        let refused = self.drag_came_over(&no_copy);
        self.drag_left(&no_copy);

        self.expectations.expect(
            inside == NSDragOperation::Generic && refused == NSDragOperation::None,
            "a drag of the application itself is left to its drop target, and one another application lets nobody \
             copy is not taken",
            &format!("{inside:?}, {refused:?}"),
        );
    }

    fn mouse_event(&self, kind: NSEventType, at: Point) -> Retained<NSEvent> {
        NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
            kind,
            location(at),
            NSEventModifierFlags::empty(),
            0.0,
            self.native_window.windowNumber(),
            None,
            0,
            1,
            1.0,
        )
        .expect("the mouse event is made")
    }

    fn mouse(&self, kind: NSEventType, at: Point) {
        self.native_window.sendEvent(&self.mouse_event(kind, at));
    }

    fn a_source_hands_its_drag_to_the_system_with_its_picture(&mut self) {
        self.window.update(|scene| {
            scene.heard.drag_start = Some(DragStart {
                payload: card_payload(),
                reach: DragReach::System,
            });
        });
        self.mouse(NSEventType::LeftMouseDown, ON_CARD);
        self.mouse(NSEventType::LeftMouseDragged, DRAGGED);

        let (sources, pictures) = self
            .window
            .update(|scene| (scene.heard.dragged_sources.clone(), scene.handed_pictures.clone()));
        let card_box = Rect::new(20.0, 20.0, 80.0, 40.0);
        self.expectations.expect(
            sources == [self.card] && pictures == [(Some(card_box), 1)],
            "the window takes the drag of the card with the picture of the card alone",
            &format!("{sources:?}, {pictures:?}"),
        );

        // The system follows the mouse of the user from here on; the release it waits for is
        // put into the queue of the application.
        let released = self.mouse_event(NSEventType::LeftMouseUp, DRAGGED);
        NSApplication::sharedApplication(self.main_thread).postEvent_atStart(&released, false);
        run_main_loop(DRAG_ENDING);
        let ended = self.window.update(|scene| scene.heard.ended_drags.clone());
        self.expectations.expect(
            ended == [(self.card, DragOutcome::Cancelled)],
            "the card hears that the drag the system carried ended with no drop",
            &format!("{ended:?}"),
        );
    }
}

fn main() -> ExitCode {
    let main_thread = MainThreadMarker::new().expect("the test runs on the main thread");
    let application = Application::new().expect("this Mac runs Metal 4");
    let mut stage = Stage::open(&application, main_thread);
    run_main_loop(SETTLING);

    stage.the_view_asks_for_the_drags_its_targets_take();
    stage.a_drag_of_another_application_is_carried_over_the_tree_and_dropped();
    stage.a_drag_that_leaves_is_over_for_the_tree();
    stage.the_answer_follows_where_the_drag_began_and_what_its_source_allows();
    stage.a_source_hands_its_drag_to_the_system_with_its_picture();

    // SAFETY: a pasteboard with a unique name is released once nothing reads it any more.
    let (): () = unsafe { msg_send![&*stage.pasteboard, releaseGlobally] };
    stage.expectations.report("system drag")
}
