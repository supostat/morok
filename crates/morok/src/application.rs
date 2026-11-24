use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use morok_components::view::{Mount, Surfaces};
use morok_core::{ElementId, ImageId, ImageStatus};
use morok_platform::{Appearance, AtlasStatistics, CanvasPainter, PasteboardClipboard, RendererError, WindowOptions};
use morok_reactive::Runtime;
use morok_style::Theme;

use crate::application_state::ApplicationState;
use crate::hosted_surface::HostedSurface;
use crate::shared_canvas_painter::SharedCanvasPainter;
use crate::styles::Styles;
use crate::window::Window;

/// A running application: the event loop of the system, the GPU resources its windows share
/// and the signal graph its components live in. Every window is a surface with a tree of its
/// own; one signal may be shown in several of them.
///
/// A clone is another handle to the same application: a handler keeps one to open and close
/// windows through [`defer`](Self::defer).
#[derive(Clone, Debug)]
pub struct Application {
    platform: Rc<morok_platform::Application>,
    state: Rc<RefCell<ApplicationState>>,
    canvas_painter: SharedCanvasPainter,
}

impl Application {
    /// An application whose trees are styled by `styles`.
    ///
    /// # Panics
    ///
    /// When called from a thread other than the main one.
    pub fn new(styles: Styles) -> Result<Self, RendererError> {
        let platform = morok_platform::Application::new()?;
        let mut state = ApplicationState::new(styles, platform.renderer().clone());
        state.surfaces.register_kit_icons(|svg| {
            platform
                .renderer()
                .add_icon(svg)
                .expect("the icons of the kit are SVG the renderer reads")
        });
        Ok(Self {
            platform: Rc::new(platform),
            state: Rc::new(RefCell::new(state)),
            canvas_painter: SharedCanvasPainter::default(),
        })
    }

    /// Opens a window on a new surface and mounts `root` on it: the component is called once
    /// and the element it builds becomes the child of the root of the tree. The texts of the
    /// surface copy to and paste from the clipboard of the system. The window shares the GPU
    /// resources of the application with the windows opened before it, and closes when the
    /// value returned is dropped. Its canvases are drawn by the painter of the application.
    ///
    /// # Panics
    ///
    /// When called while the application tells a handler of an event: a handler opens a
    /// window through [`defer`](Self::defer).
    pub fn open_window(&self, options: &WindowOptions, root: impl FnOnce(&mut Mount<'_>) -> ElementId) -> Window {
        let platform = morok_platform::Window::open(&self.platform, options, |frame_scheduler, settings| {
            let state = &mut *self.state.borrow_mut();
            let theme = state.styles.theme(settings.color_scheme).clone();
            let surface = state
                .surfaces
                .open(&mut state.runtime, state.styles.style_sheet.clone(), theme);
            let tree = state.surfaces.surface_mut(surface).tree_mut();
            tree.set_reduced_motion(settings.reduces_motion);
            tree.set_frame_scheduler(frame_scheduler);
            tree.set_clipboard(Box::new(PasteboardClipboard::general()));
            let tree_root = tree.root();
            state.surfaces.mount(&mut state.runtime, surface, tree_root, root);
            HostedSurface::new(Rc::clone(&self.state), surface, Duration::ZERO)
        });
        platform.set_canvas_painter(Box::new(self.canvas_painter.clone()));
        let surface = platform.update(|content| content.surface);
        Window {
            platform,
            surface,
            application: Rc::clone(&self.platform),
            canvas_painter: self.canvas_painter.clone(),
        }
    }

    /// Names who draws the canvases of every window of the application, the windows open now
    /// among them: the painter is handed the content box of each canvas in paint order, with
    /// the values of the tokens of its surface.
    pub fn set_canvas_painter(&self, painter: impl CanvasPainter + 'static) {
        self.canvas_painter.replace(Box::new(painter));
    }

    /// Restyles every surface of the application with other values of its tokens: `light` and
    /// `dark` are the values a tokens file the application switches to gives the tokens its
    /// style sheet was compiled against. Windows opened from now on, and a change of the color
    /// scheme, take them too.
    ///
    /// # Panics
    ///
    /// When called while the application tells a handler of an event: a handler switches the
    /// values through [`defer`](Self::defer).
    pub fn set_themes(&self, light: Theme, dark: Theme) {
        let state = &mut *self.state.borrow_mut();
        state.styles.light = light;
        state.styles.dark = dark;
        let theme = state.styles.theme(self.platform.color_scheme()).clone();
        state.surfaces.set_theme(&theme);
    }

    /// Reads or writes the signals of the application from outside its components. What a
    /// write changes on the surfaces shows in their next frame.
    pub fn update<Outcome>(&self, change: impl FnOnce(&mut Runtime<Surfaces>) -> Outcome) -> Outcome {
        let state = &mut *self.state.borrow_mut();
        let outcome = change(&mut state.runtime);
        if state.runtime.has_queued_effects() {
            state.surfaces.request_frames();
        }
        outcome
    }

    /// Runs `work` once the event the application is handling is over. A handler of a
    /// component is called in the middle of an event, where no window can be opened or
    /// dropped; it hands what opens or drops one to this instead.
    pub fn defer(&self, work: impl FnOnce() + 'static) {
        self.platform.run_soon(work);
    }

    /// Takes an encoded picture — PNG, JPEG, HEIC or any format the system reads — and decodes
    /// it off the main thread. An [`Image`](morok_components::Image) shows it by the id: at the
    /// size of its box while it decodes, then at its natural size unless its style sizes it.
    /// The bytes are the one way in: the application reads the files itself.
    pub fn add_image(&self, encoded: Vec<u8>) -> ImageId {
        self.platform.renderer().add_image(encoded)
    }

    /// How far the decoder has come with a picture the application added.
    ///
    /// # Panics
    ///
    /// When the id names no picture the application added.
    pub fn image_status(&self, image: ImageId) -> ImageStatus {
        self.platform.renderer().image_status(image)
    }

    /// The state of the glyph atlases the windows of the application share.
    pub fn atlas_statistics(&self) -> AtlasStatistics {
        self.platform.renderer().atlas_statistics()
    }

    /// Shows the application in the given appearance, or lets it follow the system again.
    pub fn set_appearance(&self, appearance: Appearance) {
        self.platform.set_appearance(appearance);
    }

    /// Runs the event loop until the application quits.
    pub fn run(&self) {
        self.platform.run();
    }

    /// Runs the event loop for `duration` and returns.
    pub fn run_for(&self, duration: Duration) {
        self.platform.run_for(duration);
    }
}
