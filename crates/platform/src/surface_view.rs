use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use morok_core::{DisplayList, FrameScheduler, Size};
use morok_style::ColorScheme;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{NSAppearanceCustomization, NSResponder, NSView};
use objc2_core_foundation::CGSize;
use objc2_core_graphics::{CGColorSpace, kCGColorSpaceSRGB};
use objc2_foundation::{NSRect, NSSize};
use objc2_metal::{MTLDrawable, MTLResidencySet};
use objc2_quartz_core::{CADisplayLink, CALayer, CAMetalDrawable, CAMetalLayer};

use crate::appearance::color_scheme_of;
use crate::frame_loop::FrameLoop;
use crate::frame_requester::FrameRequester;
use crate::frame_statistics::FrameStatistics;
use crate::render::frame_renderer::FrameRenderer;
use crate::render::renderer::{PIXEL_FORMAT, Renderer};
use crate::render::screen_target::ScreenTarget;
use crate::render::target_geometry::TargetGeometry;
use crate::surface_content::SurfaceContent;

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
    color_scheme: Cell<ColorScheme>,
    /// Whether the screen no longer shows the last display list: the surface was resized,
    /// moved to a display of another scale, or never drawn.
    surface_is_stale: Cell<bool>,
    statistics: Cell<FrameStatistics>,
}

define_class!(
    /// The view of a surface: a Metal layer the display lists of its content are drawn into,
    /// and the display link that paces the frames.
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
        fn display_link_did_fire(&self, _display_link: &CADisplayLink) {
            self.run_frame();
        }
    }
);

impl SurfaceView {
    /// A view that shows nothing yet: the window gives it its content before it shows it.
    pub(crate) fn new(
        main_thread: MainThreadMarker,
        renderer: &Renderer,
        frame: NSRect,
        color_scheme: ColorScheme,
    ) -> Retained<Self> {
        let layer = CAMetalLayer::new();
        layer.setDevice(Some(renderer.device()));
        layer.setPixelFormat(PIXEL_FORMAT);
        layer.setFramebufferOnly(true);
        // SAFETY: the name is a constant of Core Graphics.
        layer.setColorspace(CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB })).as_deref());
        let state = SurfaceState {
            frame_renderer: RefCell::new(FrameRenderer::new(renderer, FRAMES_IN_FLIGHT)),
            drawable_residency_set: layer.residencySet(),
            layer: layer.clone(),
            frame_loop: OnceCell::new(),
            content: OnceCell::new(),
            color_scheme: Cell::new(color_scheme),
            surface_is_stale: Cell::new(true),
            statistics: Cell::new(FrameStatistics::default()),
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
        assert!(
            self.ivars().content.set(content).is_ok(),
            "a view gets its content once"
        );
    }

    /// The size of the surface in CSS pixels.
    pub(crate) fn viewport(&self) -> Size {
        self.target_geometry().viewport
    }

    pub(crate) fn statistics(&self) -> FrameStatistics {
        self.ivars().statistics.get()
    }

    /// Ends the frame loop: the display link holds the view until then.
    pub(crate) fn stop_frames(&self) {
        self.frame_loop().stop();
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

    fn content(&self) -> &Rc<RefCell<dyn SurfaceContent>> {
        self.ivars()
            .content
            .get()
            .expect("a view has its content before it enters a window")
    }

    /// One tick of the display link: the content runs its frame, and the frame is drawn when
    /// it damaged anything or the screen shows a stale surface.
    fn run_frame(&self) {
        let state = self.ivars();
        let frame_loop = self.frame_loop();
        let mut statistics = state.statistics.get();
        statistics.display_link_callbacks += 1;
        frame_loop.begin_frame();
        let geometry = self.target_geometry();
        if !geometry.is_empty() {
            let mut content = self.content().borrow_mut();
            let tree = content.frame(geometry.viewport, geometry.scale);
            if state.surface_is_stale.get() || !tree.damage().is_empty() {
                if self.present(tree.display_list(), &geometry) {
                    state.surface_is_stale.set(false);
                    statistics.presented_frames += 1;
                } else {
                    state.surface_is_stale.set(true);
                    frame_loop.request_frame();
                }
            }
        }
        state.statistics.set(statistics);
        frame_loop.end_frame();
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
