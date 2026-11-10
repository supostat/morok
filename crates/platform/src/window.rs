use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use morok_core::{FrameScheduler, Size};
use objc2::MainThreadOnly;
use objc2::rc::Retained;
use objc2_app_kit::{NSBackingStoreType, NSWindow, NSWindowStyleMask};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

use crate::application::Application;
use crate::canvas::canvas_painter::CanvasPainter;
use crate::frame_statistics::FrameStatistics;
use crate::surface_content::SurfaceContent;
use crate::surface_settings::SurfaceSettings;
use crate::surface_view::SurfaceView;
use crate::window_options::WindowOptions;

/// A window of the system showing one surface. Closing follows the value: the window is on
/// screen for as long as it lives.
pub struct Window<Content> {
    native: Retained<NSWindow>,
    view: Retained<SurfaceView>,
    content: Rc<RefCell<Content>>,
}

impl<Content: SurfaceContent + 'static> Window<Content> {
    /// Opens a window on the content `build_content` returns. The content is given the frame
    /// scheduler of the window, for its tree to ask for frames through, and the settings of the
    /// system the window opens under.
    pub fn open(
        application: &Application,
        options: &WindowOptions,
        build_content: impl FnOnce(Box<dyn FrameScheduler>, SurfaceSettings) -> Content,
    ) -> Self {
        let main_thread = application.main_thread();
        let settings = application.surface_settings();
        let content_rect = NSRect::new(NSPoint::ZERO, ns_size(options.content_size));
        let view = SurfaceView::new(main_thread, application.renderer(), content_rect, settings);
        let content = Rc::new(RefCell::new(build_content(view.frame_scheduler(), settings)));
        view.set_content(Rc::clone(&content) as Rc<RefCell<dyn SurfaceContent>>);

        let style = NSWindowStyleMask::Titled
            | NSWindowStyleMask::Closable
            | NSWindowStyleMask::Miniaturizable
            | NSWindowStyleMask::Resizable;
        // SAFETY: the window is created on the main thread, and it is not released when closed:
        // this value owns it.
        let native = unsafe {
            let native = NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(main_thread),
                content_rect,
                style,
                NSBackingStoreType::Buffered,
                false,
            );
            native.setReleasedWhenClosed(false);
            native
        };
        native.setTitle(&NSString::from_str(&options.title));
        native.setContentView(Some(&view));
        native.center();
        native.makeKeyAndOrderFront(None);
        Self { native, view, content }
    }

    /// Reads or changes the content. A change its tree notices asks for a frame by itself.
    pub fn update<Outcome>(&self, change: impl FnOnce(&mut Content) -> Outcome) -> Outcome {
        change(&mut self.content.borrow_mut())
    }
}

impl<Content> Window<Content> {
    /// The size of the surface in CSS pixels.
    pub fn viewport(&self) -> Size {
        self.view.viewport()
    }

    /// Resizes the window so that its surface takes `content_size` CSS pixels.
    pub fn set_content_size(&self, content_size: Size) {
        self.native.setContentSize(ns_size(content_size));
    }

    pub fn frame_statistics(&self) -> FrameStatistics {
        self.view.statistics()
    }

    /// Names who draws the canvases of the window.
    pub fn set_canvas_painter(&self, painter: Box<dyn CanvasPainter>) {
        self.view.set_canvas_painter(painter);
    }
}

impl<Content> Debug for Window<Content> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("Window")
            .field("viewport", &self.viewport())
            .field("frame_statistics", &self.frame_statistics())
            .finish_non_exhaustive()
    }
}

impl<Content> Drop for Window<Content> {
    fn drop(&mut self) {
        self.view.close();
        self.native.close();
    }
}

fn ns_size(size: Size) -> NSSize {
    NSSize::new(f64::from(size.width), f64::from(size.height))
}
