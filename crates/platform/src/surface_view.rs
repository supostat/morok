mod key_routing;
mod text_input;

use std::cell::{Cell, OnceCell, RefCell};
use std::ffi::c_void;
use std::rc::Rc;
use std::time::Duration;

use morok_core::{
    DisplayList, ElementTree, FrameScheduler, KeyPhase, PointerButton, PointerInputKind, Size, TextCommand,
};
use morok_style::ColorScheme;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObjectProtocol, ProtocolObject, Sel};
use objc2::{AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSAppearanceCustomization, NSEvent, NSResponder, NSTextInputClient, NSTrackingArea, NSTrackingAreaOptions, NSView,
    NSWindowDidBecomeKeyNotification, NSWindowDidResignKeyNotification, NSWorkspace,
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
use crate::frame_loop::FrameLoop;
use crate::frame_requester::FrameRequester;
use crate::frame_statistics::FrameStatistics;
use crate::input_translation;
use crate::render::frame_renderer::FrameRenderer;
use crate::render::renderer::{PIXEL_FORMAT, Renderer};
use crate::render::screen_target::ScreenTarget;
use crate::render::target_geometry::TargetGeometry;
use crate::surface_content::SurfaceContent;
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
}

define_class!(
    /// The view of a surface: a Metal layer the display lists of its content are drawn into,
    /// the display link that paces the frames, the responder that takes the mouse and the
    /// keyboard for its content, and the accessibility element whose children are the nodes
    /// of the accessibility tree of the content.
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
            self.pointer_released(event, PointerButton::Primary);
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
            self.pointer_input(event, PointerInputKind::Moved);
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
    /// follows the settings of the system from `settings` on.
    pub(crate) fn new(
        main_thread: MainThreadMarker,
        renderer: &Renderer,
        frame: NSRect,
        settings: SurfaceSettings,
    ) -> Retained<Self> {
        let layer = CAMetalLayer::new();
        layer.setDevice(Some(renderer.device()));
        layer.setPixelFormat(PIXEL_FORMAT);
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

    pub(crate) fn set_canvas_painter(&self, painter: Box<dyn CanvasPainter>) {
        self.ivars().frame_renderer.borrow_mut().set_canvas_painter(painter);
    }

    pub(crate) fn statistics(&self) -> FrameStatistics {
        self.ivars().statistics.get()
    }

    /// The window closes: the display link, which holds the view, and the notifications of the
    /// system let go of it.
    pub(crate) fn close(&self) {
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

    /// Returns whether the content used the key.
    fn key_input(&self, event: &NSEvent, phase: KeyPhase) -> bool {
        input_translation::key_event(event, phase)
            .is_some_and(|key_event| self.content().borrow_mut().key_input(&key_event))
    }

    /// A window got or lost the keyboard; the accessibility of the view follows its own window.
    fn window_key_state_changed(&self, notification: &NSNotification, is_key: bool) {
        let Some(window) = self.window() else {
            return;
        };
        let is_of_this_window = notification.object().is_some_and(|object| {
            Retained::as_ptr(&object).cast::<c_void>() == Retained::as_ptr(&window).cast::<c_void>()
        });
        if is_of_this_window {
            self.accessibility().view_focus_changed(is_key);
        }
    }

    fn accessibility(&self) -> &SurfaceAccessibility {
        self.ivars()
            .accessibility
            .get()
            .expect("a view has its content before it enters a window")
    }

    fn content(&self) -> &Rc<RefCell<dyn SurfaceContent>> {
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
                if self.present(tree.display_list(), &geometry) {
                    state.surface_is_stale.set(false);
                    statistics.presented_frames += 1;
                } else {
                    state.surface_is_stale.set(true);
                    frame_loop.request_frame();
                }
            };
            self.content()
                .borrow_mut()
                .frame(geometry.viewport, geometry.scale, now, &mut show);
        }
        state.statistics.set(statistics);
        frame_loop.end_frame();
        self.accessibility().frame_ran();
    }

    /// Draws the display list into the next drawable of the layer. Returns whether the layer
    /// had one to give; a layer that is not on screen may have none.
    fn present(&self, display_list: &DisplayList, geometry: &TargetGeometry) -> bool {
        let state = self.ivars();
        let Some(drawable) = state.layer.nextDrawable() else {
            return false;
        };
        state.frame_renderer.borrow_mut().render(
            display_list,
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
