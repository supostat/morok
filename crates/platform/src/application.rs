use morok_style::ColorScheme;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};

use crate::appearance::{Appearance, color_scheme_of};
use crate::render::renderer::Renderer;
use crate::renderer_error::RendererError;

/// The running application: the event loop of `AppKit` and the GPU resources its windows share.
#[derive(Debug)]
pub struct Application {
    native: Retained<NSApplication>,
    main_thread: MainThreadMarker,
    renderer: Renderer,
}

impl Application {
    /// # Panics
    ///
    /// When called from a thread other than the main one: `AppKit` lives there.
    pub fn new() -> Result<Self, RendererError> {
        let main_thread = MainThreadMarker::new().expect("an application is created on the main thread");
        let native = NSApplication::sharedApplication(main_thread);
        native.setActivationPolicy(NSApplicationActivationPolicy::Regular);
        Ok(Self {
            native,
            main_thread,
            renderer: Renderer::new()?,
        })
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    /// Shows the application in the given appearance, or lets it follow the system again.
    /// Every window is told when its color scheme changes with it.
    pub fn set_appearance(&self, appearance: Appearance) {
        self.native.setAppearance(appearance.system_appearance().as_deref());
    }

    /// The color scheme the application is shown in now.
    pub fn color_scheme(&self) -> ColorScheme {
        color_scheme_of(&self.native.effectiveAppearance())
    }

    /// Runs the event loop until the application quits.
    pub fn run(&self) {
        self.native.run();
    }

    pub(crate) fn main_thread(&self) -> MainThreadMarker {
        self.main_thread
    }
}
