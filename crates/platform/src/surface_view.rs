mod dragging;
mod key_routing;
mod text_input;

use std::cell::{Cell, OnceCell, RefCell};
use std::ffi::c_void;
use std::rc::Rc;
use std::time::Duration;

use dispatch2::{DispatchQueue, MainThreadBound};
use morok_core::{
    DisplayList, ElementTree, FrameScheduler, KeyPhase, PayloadType, Point, PointerButton, PointerInputKind, Size,
    TextCommand,
};
use morok_style::{ColorScheme, Theme};
use objc2::rc::{Retained, Weak};
use objc2::runtime::{AnyObject, NSObjectProtocol, ProtocolObject, Sel};
use objc2::{AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send, sel};
use objc2_app_kit::{
    NSAppearanceCustomization, NSApplication, NSApplicationDidResignActiveNotification, NSDragOperation,
    NSDraggingContext, NSDraggingDestination, NSDraggingInfo, NSDraggingSession, NSDraggingSource, NSEvent,
    NSResponder, NSTextInputClient, NSTrackingArea, NSTrackingAreaOptions, NSView, NSWindow,
    NSWindowDidBecomeKeyNotification, NSWindowDidMoveNotification, NSWindowDidResignKeyNotification, NSWorkspace,
    NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification,
};
use objc2_core_foundation::CGSize;
use objc2_core_graphics::{CGColorSpace, kCGColorSpaceSRGB};
use objc2_foundation::{
    NSArray, NSAttributedString, NSAttributedStringKey, NSNotification, NSNotificationCenter, NSPoint, NSRange, NSRect,
    NSSize, NSUInteger,
};
use objc2_metal::{MTLDrawable, MTLResidencySet};
use objc2_quartz_core::{CADisplayLink, CALayer, CAMetalDrawable, CAMetalLayer};

use crate::accessibility::surface_accessibility::SurfaceAccessibility;
use crate::appearance::color_scheme_of;
use crate::canvas::canvas_painter::CanvasPainter;
use crate::drag_picture::DragPicture;
use crate::frame_image::FrameImage;
use crate::frame_loop::FrameLoop;
use crate::frame_requester::FrameRequester;
use crate::frame_statistics::FrameStatistics;
use crate::input_translation;
use crate::menu_bar;
use crate::menu_bar::menu_bar_target::MenuBarTarget;
use crate::native_menu;
use crate::overlay::overlay_request::OverlayRequest;
use crate::overlay::screen_space::ScreenSpace;
use crate::overlay::shown_overlay::ShownOverlay;
use crate::render::frame_renderer::FrameRenderer;
use crate::render::renderer::{PIXEL_FORMAT, Renderer};
use crate::render::screen_target::ScreenTarget;
use crate::render::target_geometry::TargetGeometry;
use crate::surface_content::SurfaceContent;
use crate::surface_key::SurfaceKey;
use crate::surface_settings::SurfaceSettings;
use key_routing::KeyRouting;

/// A layer hands out three drawables, so three frames may be on their way to the screen.
const FRAMES_IN_FLIGHT: usize = 3;

pub(crate) struct SurfaceState {
    layer: Retained<CAMetalLayer>,
    frame_renderer: RefCell<FrameRenderer>,
    /// Holds the textures of the drawables of the layer.
    drawable_residency_set: Retained<ProtocolObject<dyn MTLResidencySet>>,
    /// Set once the view exists: the display link calls the view.
    frame_loop: OnceCell<Rc<FrameLoop>>,
    /// Set by the window before the view enters it.
    content: OnceCell<Rc<RefCell<dyn SurfaceContent>>>,
    /// Set with the content, whose accessibility tree it shows.
    accessibility: OnceCell<SurfaceAccessibility>,
    color_scheme: Cell<ColorScheme>,
    reduces_motion: Cell<bool>,
    /// Whether the screen no longer shows the last display list: the surface was resized,
    /// moved to a display of another scale, or never drawn.
    surface_is_stale: Cell<bool>,
    statistics: Cell<FrameStatistics>,
    /// The key being handed to the input methods.
    key_in_flight: RefCell<Option<Retained<NSEvent>>>,
    key_routing: Cell<KeyRouting>,
    /// The GPU resources of the application, for the views of the overlays of this one.
    renderer: Renderer,
    /// The room the screen has for the surface when the surface is an overlay: its content is
    /// laid out in that room, and the window takes the size the content comes to.
    overlay_room: Cell<Option<Size>>,
    /// The overlays the content asked for, on screen.
    overlays: RefCell<Vec<ShownOverlay>>,
    /// Scratch space for what the content asks for after a frame.
    overlay_requests: RefCell<Vec<OverlayRequest>>,
    /// What the lines of the menu bar tell of their choice while the bar shows the menus of
    /// the content.
    menu_bar_target: RefCell<Option<Retained<MenuBarTarget>>>,
    /// The revision of the menus of the content the menu bar was built from last.
    shown_menu_bar: Cell<Option<u64>>,
    /// The payload types the content takes drags of the system in, as the system was asked to
    /// offer them to the view last.
    drop_types: RefCell<Vec<PayloadType>>,
    /// The revision of the payload types of the content the system was asked with last.
    registered_drop_types: Cell<Option<u64>>,
    /// The picture of the last drag the view began in the system.
    handed_picture: RefCell<Option<DragPicture>>,
}

define_class!(
    /// The view of a surface: a Metal layer the display lists of its content are drawn into,
    /// the display link that paces the frames, the responder that takes the mouse and the
    /// keyboard for its content, the source and the destination of the drags of the system
    /// its content begins and takes, and the accessibility element whose children are the
    /// nodes of the accessibility tree of the content.
    #[unsafe(super(NSView, NSResponder))]
    #[thread_kind = MainThreadOnly]
    #[ivars = SurfaceState]
    pub(crate) struct SurfaceView;

    impl SurfaceView {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }

        #[unsafe(method(setFrameSize:))]
        fn set_frame_size(&self, size: NSSize) {
            // SAFETY: the superclass implements the method with this signature.
            let (): () = unsafe { msg_send![super(self), setFrameSize: size] };
            self.surface_changed();
        }

        #[unsafe(method(viewDidChangeBackingProperties))]
        fn view_did_change_backing_properties(&self) {
            // SAFETY: the superclass implements the method with this signature.
            let (): () = unsafe { msg_send![super(self), viewDidChangeBackingProperties] };
            self.surface_changed();
        }

        #[unsafe(method(viewDidChangeEffectiveAppearance))]
        fn view_did_change_effective_appearance(&self) {
            // SAFETY: the superclass implements the method with this signature.
            let (): () = unsafe { msg_send![super(self), viewDidChangeEffectiveAppearance] };
            self.appearance_changed();
        }

        #[unsafe(method(displayLinkDidFire:))]
        fn display_link_did_fire(&self, display_link: &CADisplayLink) {
            self.run_frame(Duration::from_secs_f64(display_link.targetTimestamp()));
        }

        #[unsafe(method(accessibilityDisplayOptionsDidChange:))]
        fn accessibility_display_options_did_change(&self, _notification: &NSNotification) {
            self.motion_setting_changed();
        }

        #[unsafe(method(windowDidBecomeKey:))]
        fn window_did_become_key(&self, notification: &NSNotification) {
            self.window_key_state_changed(notification, true);
            self.keyboard_went_to(notification);
        }

        #[unsafe(method(applicationDidResignActive:))]
        fn application_did_resign_active(&self, _notification: &NSNotification) {
            self.user_turned_away();
        }

        #[unsafe(method(windowDidMove:))]
        fn window_did_move(&self, notification: &NSNotification) {
            self.window_moved(notification);
        }

        #[unsafe(method(windowDidResignKey:))]
        fn window_did_resign_key(&self, notification: &NSNotification) {
            self.window_key_state_changed(notification, false);
        }

        #[unsafe(method(accessibilityChildren))]
        fn accessibility_children(&self) -> *mut AnyObject {
            self.accessibility().children()
        }

        #[unsafe(method(accessibilityFocusedUIElement))]
        fn accessibility_focused_ui_element(&self) -> *mut AnyObject {
            self.accessibility().focused_element()
        }

        #[unsafe(method(accessibilityHitTest:))]
        fn accessibility_hit_test(&self, screen_point: NSPoint) -> *mut AnyObject {
            self.accessibility().element_at(screen_point)
        }

        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool {
            true
        }

        /// The click that brings an inactive window forward also presses what it lands on,
        /// as it does on a button of the system.
        #[unsafe(method(acceptsFirstMouse:))]
        fn accepts_first_mouse(&self, _event: Option<&NSEvent>) -> bool {
            true
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, event: &NSEvent) {
            self.pointer_pressed(event, PointerButton::Primary);
        }

        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, event: &NSEvent) {
            let click_count = input_translation::click_count(event);
            self.held_pointer_input(
                event,
                PointerInputKind::Released {
                    button: PointerButton::Primary,
                    click_count,
                },
            );
        }

        #[unsafe(method(rightMouseDown:))]
        fn right_mouse_down(&self, event: &NSEvent) {
            self.pointer_pressed(event, PointerButton::Secondary);
        }

        #[unsafe(method(rightMouseUp:))]
        fn right_mouse_up(&self, event: &NSEvent) {
            self.pointer_released(event, PointerButton::Secondary);
        }

        #[unsafe(method(otherMouseDown:))]
        fn other_mouse_down(&self, event: &NSEvent) {
            self.pointer_pressed(event, PointerButton::Auxiliary);
        }

        #[unsafe(method(otherMouseUp:))]
        fn other_mouse_up(&self, event: &NSEvent) {
            self.pointer_released(event, PointerButton::Auxiliary);
        }

        #[unsafe(method(mouseMoved:))]
        fn mouse_moved(&self, event: &NSEvent) {
            self.pointer_input(event, PointerInputKind::Moved);
        }

        #[unsafe(method(mouseDragged:))]
        fn mouse_dragged(&self, event: &NSEvent) {
            self.held_pointer_input(event, PointerInputKind::Moved);
            self.begin_requested_drag(event);
        }

        #[unsafe(method(rightMouseDragged:))]
        fn right_mouse_dragged(&self, event: &NSEvent) {
            self.pointer_input(event, PointerInputKind::Moved);
        }

        #[unsafe(method(otherMouseDragged:))]
        fn other_mouse_dragged(&self, event: &NSEvent) {
            self.pointer_input(event, PointerInputKind::Moved);
        }

        #[unsafe(method(mouseEntered:))]
        fn mouse_entered(&self, event: &NSEvent) {
            self.pointer_input(event, PointerInputKind::Moved);
        }

        #[unsafe(method(mouseExited:))]
        fn mouse_exited(&self, event: &NSEvent) {
            self.pointer_input(event, PointerInputKind::Exited);
        }

        #[unsafe(method(scrollWheel:))]
        fn scroll_wheel(&self, event: &NSEvent) {
            if let Some(input) = input_translation::scroll_input(self, event) {
                self.content().borrow_mut().scroll_input(&input);
            }
        }

        /// While the content edits text, a key goes through the input methods of the system
        /// first: they type, compose and dictate, and hand back the keys that are commands.
        #[unsafe(method(keyDown:))]
        fn key_down(&self, event: &NSEvent) {
            let edits_text = self.content().borrow_mut().focused_text().is_some();
            let was_used = if edits_text {
                self.interpret_key(event)
            } else {
                self.key_input(event, KeyPhase::Pressed)
            };
            if !was_used {
                // SAFETY: the superclass implements the method with this signature; it hands
                // the key on along the responder chain.
                let (): () = unsafe { msg_send![super(self), keyDown: event] };
            }
        }

        #[unsafe(method(keyUp:))]
        fn key_up(&self, event: &NSEvent) {
            if !self.key_input(event, KeyPhase::Released) {
                // SAFETY: the superclass implements the method with this signature; it hands
                // the key on along the responder chain.
                let (): () = unsafe { msg_send![super(self), keyUp: event] };
            }
        }

        #[unsafe(method(copy:))]
        fn copy_text(&self, _sender: Option<&AnyObject>) {
            self.input_text_command(TextCommand::Copy);
        }

        #[unsafe(method(cut:))]
        fn cut_text(&self, _sender: Option<&AnyObject>) {
            self.input_text_command(TextCommand::Cut);
        }

        #[unsafe(method(paste:))]
        fn paste_text(&self, _sender: Option<&AnyObject>) {
            self.input_text_command(TextCommand::Paste);
        }

        #[unsafe(method(selectAll:))]
        fn select_all_text(&self, _sender: Option<&AnyObject>) {
            self.input_text_command(TextCommand::SelectAll);
        }

        #[unsafe(method(undo:))]
        fn undo_text(&self, _sender: Option<&AnyObject>) {
            self.input_text_command(TextCommand::Undo);
        }

        #[unsafe(method(redo:))]
        fn redo_text(&self, _sender: Option<&AnyObject>) {
            self.input_text_command(TextCommand::Redo);
        }
    }

    unsafe impl NSObjectProtocol for SurfaceView {}

    unsafe impl NSDraggingDestination for SurfaceView {
        #[unsafe(method(draggingEntered:))]
        fn dragging_entered(&self, sender: &ProtocolObject<dyn NSDraggingInfo>) -> NSDragOperation {
            self.drag_came_over(sender)
        }

        #[unsafe(method(draggingUpdated:))]
        fn dragging_updated(&self, sender: &ProtocolObject<dyn NSDraggingInfo>) -> NSDragOperation {
            self.drag_moved_over(sender)
        }

        #[unsafe(method(draggingExited:))]
        fn dragging_exited(&self, _sender: Option<&ProtocolObject<dyn NSDraggingInfo>>) {
            self.drag_left();
        }

        #[unsafe(method(performDragOperation:))]
        fn perform_drag_operation(&self, sender: &ProtocolObject<dyn NSDraggingInfo>) -> bool {
            self.drag_dropped(sender)
        }

        #[unsafe(method(draggingEnded:))]
        fn dragging_ended(&self, _sender: &ProtocolObject<dyn NSDraggingInfo>) {
            self.drag_left();
        }

        /// The view hears of a drag when it moves: a drag at rest changes nothing.
        #[unsafe(method(wantsPeriodicDraggingUpdates))]
        fn wants_periodic_dragging_updates(&self) -> bool {
            false
        }
    }

    unsafe impl NSDraggingSource for SurfaceView {
        #[unsafe(method(draggingSession:sourceOperationMaskForDraggingContext:))]
        fn dragging_session_source_operation_mask(
            &self,
            _session: &NSDraggingSession,
            context: NSDraggingContext,
        ) -> NSDragOperation {
            Self::offered_operations(context)
        }

        #[unsafe(method(draggingSession:endedAtPoint:operation:))]
        fn dragging_session_ended(&self, _session: &NSDraggingSession, _screen_point: NSPoint, operation: NSDragOperation) {
            self.handed_drag_ended(operation);
        }
    }

    unsafe impl NSTextInputClient for SurfaceView {
        #[unsafe(method(insertText:replacementRange:))]
        fn insert_text_replacement_range(&self, string: &AnyObject, replacement_range: NSRange) {
            self.input_insert(string, replacement_range);
        }

        #[unsafe(method(doCommandBySelector:))]
        fn do_command_by_selector(&self, _selector: Sel) {
            self.input_command();
        }

        #[unsafe(method(setMarkedText:selectedRange:replacementRange:))]
        fn set_marked_text_selected_range_replacement_range(
            &self,
            string: &AnyObject,
            selected_range: NSRange,
            replacement_range: NSRange,
        ) {
            self.input_set_marked(string, selected_range, replacement_range);
        }

        #[unsafe(method(unmarkText))]
        fn unmark_text(&self) {
            self.input_unmark();
        }

        #[unsafe(method(selectedRange))]
        fn selected_range(&self) -> NSRange {
            self.input_selected_range()
        }

        #[unsafe(method(markedRange))]
        fn marked_range(&self) -> NSRange {
            self.input_marked_range()
        }

        #[unsafe(method(hasMarkedText))]
        fn has_marked_text(&self) -> bool {
            self.input_has_marked_text()
        }

        #[unsafe(method_id(attributedSubstringForProposedRange:actualRange:))]
        fn attributed_substring_for_proposed_range(
            &self,
            range: NSRange,
            actual_range: *mut NSRange,
        ) -> Option<Retained<NSAttributedString>> {
            self.input_substring(range, actual_range)
        }

        #[unsafe(method_id(validAttributesForMarkedText))]
        fn valid_attributes_for_marked_text(&self) -> Retained<NSArray<NSAttributedStringKey>> {
            Self::input_valid_attributes()
        }

        #[unsafe(method(firstRectForCharacterRange:actualRange:))]
        fn first_rect_for_character_range(&self, range: NSRange, actual_range: *mut NSRange) -> NSRect {
            self.input_first_rect(range, actual_range)
        }

        #[unsafe(method(characterIndexForPoint:))]
        fn character_index_for_point(&self, point: NSPoint) -> NSUInteger {
            self.input_character_index(point)
        }
    }
);

impl SurfaceView {
    /// A view that shows nothing yet: the window gives it its content before it shows it. It
    /// follows the settings of the system from `settings` on. The view of an overlay is given
    /// the room the screen has for the overlay. A view that is not opaque shows what lies
    /// under it wherever its content draws nothing.
    pub(crate) fn new(
        main_thread: MainThreadMarker,
        renderer: &Renderer,
        frame: NSRect,
        settings: SurfaceSettings,
        overlay_room: Option<Size>,
        is_opaque: bool,
    ) -> Retained<Self> {
        let layer = CAMetalLayer::new();
        layer.setDevice(Some(renderer.device()));
        layer.setPixelFormat(PIXEL_FORMAT);
        layer.setOpaque(is_opaque);
        // A backdrop filter reads what the surface already shows.
        layer.setFramebufferOnly(false);
        // SAFETY: the name is a constant of Core Graphics.
        layer.setColorspace(CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB })).as_deref());
        let state = SurfaceState {
            frame_renderer: RefCell::new(FrameRenderer::new(renderer, FRAMES_IN_FLIGHT)),
            drawable_residency_set: layer.residencySet(),
            layer: layer.clone(),
            frame_loop: OnceCell::new(),
            content: OnceCell::new(),
            accessibility: OnceCell::new(),
            color_scheme: Cell::new(settings.color_scheme),
            reduces_motion: Cell::new(settings.reduces_motion),
            surface_is_stale: Cell::new(true),
            statistics: Cell::new(FrameStatistics::default()),
            key_in_flight: RefCell::new(None),
            key_routing: Cell::new(KeyRouting::Waiting),
            renderer: renderer.clone(),
            overlay_room: Cell::new(overlay_room),
            menu_bar_target: RefCell::new(None),
            shown_menu_bar: Cell::new(None),
            drop_types: RefCell::new(Vec::new()),
            registered_drop_types: Cell::new(None),
            handed_picture: RefCell::new(None),
            overlays: RefCell::new(Vec::new()),
            overlay_requests: RefCell::new(Vec::new()),
        };
        let view = Self::alloc(main_thread).set_ivars(state);
        // SAFETY: the superclass initializes a view with a frame.
        let view: Retained<Self> = unsafe { msg_send![super(view), initWithFrame: frame] };
        let backing_layer: &CALayer = &layer;
        view.setLayer(Some(backing_layer));
        view.setWantsLayer(true);
        // SAFETY: the view implements the selector, taking the display link.
        let display_link = unsafe { view.displayLinkWithTarget_selector(&view, sel!(displayLinkDidFire:)) };
        view.ivars()
            .frame_loop
            .set(Rc::new(FrameLoop::new(display_link)))
            .expect("a view gets its frame loop once");
        let observer: &AnyObject = &view;
        // SAFETY: the view implements the selectors, each taking the notification, and it
        // stops observing when its window closes. The names are constants of `AppKit`.
        unsafe {
            NSWorkspace::sharedWorkspace()
                .notificationCenter()
                .addObserver_selector_name_object(
                    observer,
                    sel!(accessibilityDisplayOptionsDidChange:),
                    Some(NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification),
                    None,
                );
            let window_notifications = NSNotificationCenter::defaultCenter();
            window_notifications.addObserver_selector_name_object(
                observer,
                sel!(windowDidBecomeKey:),
                Some(NSWindowDidBecomeKeyNotification),
                None,
            );
            window_notifications.addObserver_selector_name_object(
                observer,
                sel!(windowDidResignKey:),
                Some(NSWindowDidResignKeyNotification),
                None,
            );
            window_notifications.addObserver_selector_name_object(
                observer,
                sel!(applicationDidResignActive:),
                Some(NSApplicationDidResignActiveNotification),
                None,
            );
            window_notifications.addObserver_selector_name_object(
                observer,
                sel!(windowDidMove:),
                Some(NSWindowDidMoveNotification),
                None,
            );
        }
        // The area follows the visible part of the view by itself, so its rectangle is not
        // read. The pointer is followed in the active application only, as controls of the
        // system follow it.
        let tracking_options = NSTrackingAreaOptions::MouseEnteredAndExited
            | NSTrackingAreaOptions::MouseMoved
            | NSTrackingAreaOptions::ActiveInActiveApp
            | NSTrackingAreaOptions::InVisibleRect;
        // SAFETY: the owner is the view, which implements the mouse methods the area calls, and
        // no user info is passed.
        let tracking_area = unsafe {
            NSTrackingArea::initWithRect_options_owner_userInfo(
                NSTrackingArea::alloc(),
                NSRect::ZERO,
                tracking_options,
                Some(observer),
                None,
            )
        };
        view.addTrackingArea(&tracking_area);
        view.surface_changed();
        view
    }

    /// The port through which the tree the view shows asks for frames.
    pub(crate) fn frame_scheduler(&self) -> Box<dyn FrameScheduler> {
        Box::new(FrameRequester {
            frame_loop: Rc::clone(self.frame_loop()),
        })
    }

    /// # Panics
    ///
    /// When the view already has its content.
    pub(crate) fn set_content(&self, content: Rc<RefCell<dyn SurfaceContent>>) {
        let is_in_key_window = self.window().is_some_and(|window| window.isKeyWindow());
        let accessibility = SurfaceAccessibility::new(self, is_in_key_window, Rc::clone(&content));
        let state = self.ivars();
        assert!(
            state.content.set(content).is_ok() && state.accessibility.set(accessibility).is_ok(),
            "a view gets its content once"
        );
    }

    /// The size of the surface in CSS pixels.
    pub(crate) fn viewport(&self) -> Size {
        self.target_geometry().viewport
    }

    /// Device pixels per CSS pixel on the screen the surface is shown on.
    pub(crate) fn scale(&self) -> f32 {
        self.target_geometry().scale
    }

    pub(crate) fn set_canvas_painter(&self, painter: Box<dyn CanvasPainter>) {
        self.ivars().frame_renderer.borrow_mut().set_canvas_painter(painter);
    }

    pub(crate) fn statistics(&self) -> FrameStatistics {
        self.ivars().statistics.get()
    }

    /// The pixels of the picture of the last drag the view began in the system, read back from
    /// the image the dragging session was given.
    pub(crate) fn handed_drag_picture(&self) -> Option<FrameImage> {
        self.ivars().handed_picture.borrow().as_ref()?.read_back()
    }

    pub(crate) fn renderer(&self) -> &Renderer {
        &self.ivars().renderer
    }

    /// The room the screen has for the overlay the view shows changed.
    pub(crate) fn set_overlay_room(&self, room: Size) {
        let state = self.ivars();
        if state.overlay_room.replace(Some(room)) != Some(room) {
            self.frame_loop().request_frame();
        }
    }

    /// The window closes: its overlays close with it, and the display link, which holds the
    /// view, and the notifications of the system let go of it.
    pub(crate) fn close(&self) {
        drop(self.ivars().overlays.take());
        menu_bar::forget(self, self.mtm());
        drop(self.ivars().menu_bar_target.take());
        self.frame_loop().stop();
        let observer: &AnyObject = self;
        // SAFETY: the view was added as an observer when it was created.
        unsafe {
            NSWorkspace::sharedWorkspace()
                .notificationCenter()
                .removeObserver(observer);
            NSNotificationCenter::defaultCenter().removeObserver(observer);
        }
    }

    fn frame_loop(&self) -> &Rc<FrameLoop> {
        self.ivars()
            .frame_loop
            .get()
            .expect("a view has its frame loop from its creation on")
    }

    fn target_geometry(&self) -> TargetGeometry {
        let bounds = self.bounds();
        let scale = self.window().map_or(1.0, |window| window.backingScaleFactor());
        TargetGeometry::new(
            Size {
                width: bounds.size.width as f32,
                height: bounds.size.height as f32,
            },
            scale as f32,
        )
    }

    /// The view was resized or moved to a display of another scale: the layer takes the new
    /// size in device pixels and the whole surface is drawn again.
    fn surface_changed(&self) {
        // AppKit sizes a view while it is still being initialized, before its frame loop exists.
        let Some(frame_loop) = self.ivars().frame_loop.get() else {
            return;
        };
        let geometry = self.target_geometry();
        if !geometry.is_empty() {
            let layer = &self.ivars().layer;
            layer.setContentsScale(f64::from(geometry.scale));
            layer.setDrawableSize(CGSize {
                width: geometry.pixel_width as f64,
                height: geometry.pixel_height as f64,
            });
        }
        self.ivars().surface_is_stale.set(true);
        frame_loop.request_frame();
    }

    fn appearance_changed(&self) {
        let color_scheme = color_scheme_of(&self.effectiveAppearance());
        if self.ivars().color_scheme.replace(color_scheme) == color_scheme {
            return;
        }
        self.content().borrow_mut().color_scheme_changed(color_scheme);
    }

    fn motion_setting_changed(&self) {
        let reduces_motion = NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion();
        if self.ivars().reduces_motion.replace(reduces_motion) == reduces_motion {
            return;
        }
        self.content().borrow_mut().reduced_motion_changed(reduces_motion);
    }

    fn pointer_pressed(&self, event: &NSEvent, button: PointerButton) {
        let click_count = input_translation::click_count(event);
        self.pointer_input(event, PointerInputKind::Pressed { button, click_count });
    }

    fn pointer_released(&self, event: &NSEvent, button: PointerButton) {
        let click_count = input_translation::click_count(event);
        self.pointer_input(event, PointerInputKind::Released { button, click_count });
    }

    fn pointer_input(&self, event: &NSEvent, kind: PointerInputKind) {
        let input = input_translation::pointer_input(self, event, kind);
        self.content().borrow_mut().pointer_input(&input);
    }

    /// What the pointer did while the primary button pressed on the view is down. `AppKit`
    /// sends it here wherever the pointer is; over another surface of the application the
    /// content is told that surface and the place of the pointer on it as well.
    fn held_pointer_input(&self, event: &NSEvent, kind: PointerInputKind) {
        let input = input_translation::pointer_input(self, event, kind);
        match self.other_surface_under(event) {
            Some((under, position)) => self.content().borrow_mut().pointer_input_over(&input, under, position),
            None => self.content().borrow_mut().pointer_input(&input),
        }
    }

    /// The surface of the application topmost under the pointer of `event`, with the place of
    /// the pointer on it in its CSS pixels; none when that is the surface of this view, when
    /// no surface of the application is there, and when its content takes nothing a pointer
    /// brings from another surface. The topmost is the frontmost visible window of the
    /// application whose frame holds the point and that takes the pointer: the windows of
    /// other applications do not count.
    fn other_surface_under(&self, event: &NSEvent) -> Option<(SurfaceKey, Point)> {
        let window = self.window()?;
        let on_screen = window.convertPointToScreen(event.locationInWindow());
        let topmost = NSApplication::sharedApplication(self.mtm())
            .orderedWindows()
            .to_vec()
            .into_iter()
            .find(|candidate| {
                candidate.isVisible() && !candidate.ignoresMouseEvents() && holds(candidate.frame(), on_screen)
            })?;
        let content_view = topmost.contentView()?;
        let view = surface_view_of(&content_view)?;
        if std::ptr::eq(Retained::as_ptr(&view), self) {
            return None;
        }
        let key = view.content().borrow().surface_key()?;
        let in_view = view.convertPoint_fromView(topmost.convertPointFromScreen(on_screen), None);
        Some((
            key,
            Point {
                x: in_view.x as f32,
                y: in_view.y as f32,
            },
        ))
    }

    /// Returns whether the content used the key.
    fn key_input(&self, event: &NSEvent, phase: KeyPhase) -> bool {
        input_translation::key_event(event, phase)
            .is_some_and(|key_event| self.content().borrow_mut().key_input(&key_event))
    }

    /// A window got or lost the keyboard; the accessibility of the view follows its own window.
    fn window_key_state_changed(&self, notification: &NSNotification, is_key: bool) {
        if self.is_about_own_window(notification) {
            self.accessibility().view_focus_changed(is_key);
            if is_key {
                // The frame that follows shows the menus of the content in the menu bar.
                self.frame_loop().request_frame();
            }
        }
    }

    /// A window moved. The overlays of the view move along with its own window, and each has
    /// to be placed again: the room around its anchor changed.
    fn window_moved(&self, notification: &NSNotification) {
        if self.is_about_own_window(notification) && !self.ivars().overlays.borrow().is_empty() {
            self.frame_loop().request_frame();
        }
    }

    /// Whether the notification tells of the window the view is in.
    fn is_about_own_window(&self, notification: &NSNotification) -> bool {
        let Some(window) = self.window() else {
            return false;
        };
        notification.object().is_some_and(|object| {
            Retained::as_ptr(&object).cast::<c_void>() == Retained::as_ptr(&window).cast::<c_void>()
        })
    }

    fn accessibility(&self) -> &SurfaceAccessibility {
        self.ivars()
            .accessibility
            .get()
            .expect("a view has its content before it enters a window")
    }

    pub(crate) fn content(&self) -> &Rc<RefCell<dyn SurfaceContent>> {
        self.ivars()
            .content
            .get()
            .expect("a view has its content before it enters a window")
    }

    /// One tick of the display link for the frame shown at `now`: the content runs its frame,
    /// and the frame is drawn when it damaged anything or the screen shows a stale surface.
    fn run_frame(&self, now: Duration) {
        let state = self.ivars();
        let frame_loop = self.frame_loop();
        let mut statistics = state.statistics.get();
        statistics.display_link_callbacks += 1;
        frame_loop.begin_frame();
        let geometry = self.target_geometry();
        if !geometry.is_empty() {
            let mut show = |tree: &ElementTree| {
                if !state.surface_is_stale.get() && tree.damage().is_empty() {
                    return;
                }
                if self.present(tree.display_list(), tree.theme(), &geometry) {
                    state.surface_is_stale.set(false);
                    statistics.presented_frames += 1;
                } else {
                    state.surface_is_stale.set(true);
                    frame_loop.request_frame();
                }
            };
            let layout_viewport = state.overlay_room.get().unwrap_or(geometry.viewport);
            self.content()
                .borrow_mut()
                .frame(layout_viewport, geometry.scale, now, &mut show);
        }
        state.statistics.set(statistics);
        frame_loop.end_frame();
        self.accessibility().frame_ran();
        self.sync_overlays(now);
        self.sync_menu_bar();
        self.sync_drop_types();
        self.show_requested_native_menu();
    }

    /// Shows the menus of the content in the menu bar when they are the bar's to show: the
    /// window of the view has the keyboard, or no view has shown any menus yet. The bar is
    /// built anew only when the menus changed, or another view showed its own in between.
    fn sync_menu_bar(&self) {
        let state = self.ivars();
        let Some(window) = self.window() else {
            return;
        };
        let is_overlay = state.overlay_room.get().is_some();
        let shows_own = menu_bar::shows_menus_of(self);
        if is_overlay || !(shows_own || window.isKeyWindow() || !menu_bar::shows_menus()) {
            return;
        }
        let revision = self.content().borrow_mut().menu_bar_revision();
        if shows_own && state.shown_menu_bar.get() == Some(revision) {
            return;
        }
        let menus = self.content().borrow_mut().menu_bar();
        let target = MenuBarTarget::new(self);
        menu_bar::show(self, &menus, &target, self.mtm());
        state.menu_bar_target.replace(Some(target));
        state.shown_menu_bar.set(Some(revision));
    }

    /// Brings the windows of the overlays in line with what the content asks for after a
    /// frame: opens the new ones, places every one against its anchor, and closes the ones
    /// the content asks for no longer.
    fn sync_overlays(&self, now: Duration) {
        let Some(window) = self.window() else {
            return;
        };
        let state = self.ivars();
        let main_thread = self.mtm();
        let screen = ScreenSpace::current(main_thread);
        let room = screen.room_around(&window, main_thread);
        let scale = window.backingScaleFactor() as f32;
        let mut requests = state.overlay_requests.take();
        requests.clear();
        self.content()
            .borrow_mut()
            .overlays(room.size(), scale, now, &mut requests);
        // A window that closes hands the keyboard back, and the observers of the key window
        // hear of it at once: the overlays are not borrowed while one is dropped.
        let closed: Vec<ShownOverlay> = state
            .overlays
            .borrow_mut()
            .extract_if(.., |shown| !requests.iter().any(|request| request.id == shown.id()))
            .collect();
        drop(closed);
        for request in requests.iter().filter(|request| !request.size.is_empty()) {
            let is_shown = state.overlays.borrow().iter().any(|shown| shown.id() == request.id);
            if !is_shown {
                let opened = ShownOverlay::open(self, request, room.size(), self.settings());
                state.overlays.borrow_mut().push(opened);
            }
            let moved = state
                .overlays
                .borrow()
                .iter()
                .find(|shown| shown.id() == request.id)
                .and_then(|shown| shown.place(self, &window, request, screen, room));
            if let Some((side, origin)) = moved {
                self.content().borrow_mut().overlay_placed(request.id, side, origin);
            }
        }
        state.overlay_requests.replace(requests);
    }

    /// The settings of the system the surface follows now.
    fn settings(&self) -> SurfaceSettings {
        let state = self.ivars();
        SurfaceSettings {
            color_scheme: state.color_scheme.get(),
            reduces_motion: state.reduces_motion.get(),
        }
    }

    /// Shows the menu of the system the content asked for. The menu runs an event loop of its
    /// own until it closes, so it opens from the main queue: once the callback that found it
    /// asked for has returned and nothing is borrowed.
    fn show_requested_native_menu(&self) {
        let Some(menu) = self.content().borrow_mut().native_menu() else {
            return;
        };
        let view = MainThreadBound::new(Weak::from_retained(&self.retain()), self.mtm());
        DispatchQueue::main().exec_async(move || {
            let main_thread = MainThreadMarker::new().expect("the main queue runs on the main thread");
            let Some(view) = view.get(main_thread).load() else {
                return;
            };
            if let Some(id) = native_menu::pop_up(&view, &menu, main_thread) {
                view.content().borrow_mut().native_menu_command_chosen(id);
            }
        });
    }

    /// A window took the keyboard. When it shows no surface, the user turned to a panel of
    /// the system or to a window the kit does not draw.
    fn keyboard_went_to(&self, notification: &NSNotification) {
        let shows_surface = notification
            .object()
            .and_then(|object| object.downcast::<NSWindow>().ok())
            .and_then(|window| window.contentView())
            .is_some_and(|content_view| surface_view_of(&content_view).is_some());
        if !shows_surface {
            self.user_turned_away();
        }
    }

    /// The user left the surface for something the kit does not draw: the overlays of the
    /// surface that go on a press outside them go now.
    fn user_turned_away(&self) {
        if self.ivars().overlays.borrow().is_empty() {
            return;
        }
        self.content().borrow_mut().dismiss_overlays();
    }

    /// Draws the display list into the next drawable of the layer, its canvases under `theme`.
    /// Returns whether the layer had one to give; a layer that is not on screen may have none.
    fn present(&self, display_list: &DisplayList, theme: &Theme, geometry: &TargetGeometry) -> bool {
        let state = self.ivars();
        let Some(drawable) = state.layer.nextDrawable() else {
            return false;
        };
        state.frame_renderer.borrow_mut().render(
            display_list,
            theme,
            geometry,
            &drawable.texture(),
            Some(ScreenTarget {
                drawable: ProtocolObject::<dyn MTLDrawable>::from_ref(&*drawable),
                residency_set: &state.drawable_residency_set,
            }),
        );
        true
    }
}

/// Whether the point lies in the rectangle, on its left or bottom edge included.
fn holds(rect: NSRect, point: NSPoint) -> bool {
    point.x >= rect.origin.x
        && point.x < rect.origin.x + rect.size.width
        && point.y >= rect.origin.y
        && point.y < rect.origin.y + rect.size.height
}

/// The view of a surface the content view of a window is, or holds over the blurred backdrop
/// of the window.
fn surface_view_of(content_view: &NSView) -> Option<Retained<SurfaceView>> {
    if let Some(view) = content_view.downcast_ref::<SurfaceView>() {
        return Some(view.retain());
    }
    content_view
        .subviews()
        .to_vec()
        .into_iter()
        .find_map(|subview| subview.downcast::<SurfaceView>().ok())
}
