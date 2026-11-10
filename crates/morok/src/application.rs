use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use morok_components::view::{Mount, Surfaces};
use morok_core::ElementId;
use morok_platform::{Appearance, PasteboardClipboard, RendererError, WindowOptions};
use morok_reactive::Runtime;

use crate::application_state::ApplicationState;
use crate::styles::Styles;
use crate::window::Window;
use crate::window_content::WindowContent;

/// A running application: the event loop of the system, the GPU resources its windows share
/// and the signal graph its components live in. Every window is a surface with a tree of its
/// own; one signal may be shown in several of them.
#[derive(Debug)]
pub struct Application {
    platform: morok_platform::Application,
    state: Rc<RefCell<ApplicationState>>,
}

impl Application {
    /// An application whose trees are styled by `styles`.
    ///
    /// # Panics
    ///
    /// When called from a thread other than the main one.
    pub fn new(styles: Styles) -> Result<Self, RendererError> {
        let platform = morok_platform::Application::new()?;
        let mut state = ApplicationState::new(styles);
        state.surfaces.register_kit_icons(|svg| {
            platform
                .renderer()
                .add_icon(svg)
                .expect("the icons of the kit are SVG the renderer reads")
        });
        Ok(Self {
            platform,
            state: Rc::new(RefCell::new(state)),
        })
    }

    /// Opens a window on a new surface and mounts `root` on it: the component is called once
    /// and the element it builds becomes the child of the root of the tree. The texts of the
    /// surface copy to and paste from the clipboard of the system.
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
            WindowContent {
                state: Rc::clone(&self.state),
                surface,
            }
        });
        let surface = platform.update(|content| content.surface);
        Window { platform, surface }
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
