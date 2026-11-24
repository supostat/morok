use std::cell::{Cell, Ref, RefCell};
use std::rc::{Rc, Weak};

use morok::core::{ImageId, Size};
use morok::reactive::Signal;
use morok::{Appearance, Application, RendererError, Styles, Window, WindowOptions};

use crate::catalog_window;
use crate::disk_map::DiskMap;
use crate::gallery::Gallery;
use crate::page::Page;
use crate::styles::STYLE_SHEET;
use crate::token_set::TokenSet;
use crate::tokens::{DARK, LIGHT};

/// The size a window of the catalog opens at.
pub const WINDOW_SIZE: Size = Size {
    width: 1120.0,
    height: 720.0,
};
/// The title of the first window; the windows opened after it carry their number after it.
pub const WINDOW_TITLE: &str = "MOROK catalog";

/// The style tables of the catalog: the stylesheets of the kit and of the catalog under the
/// tokens of the catalog.
pub fn styles() -> Styles {
    Styles {
        style_sheet: STYLE_SHEET,
        light: LIGHT,
        dark: DARK,
    }
}

/// The catalog application: its windows, and the appearance and the token set they all show.
/// A clone is another handle to the same catalog; when the last handle goes, the windows of the
/// catalog close.
#[derive(Clone, Debug)]
pub struct Catalog {
    shared: Rc<Shared>,
}

/// What the handlers and the effects of the catalog reach the catalog by: a link that does not
/// keep the catalog, so that the windows, whose trees hold the handlers, do not keep themselves.
#[derive(Clone, Debug)]
pub(crate) struct CatalogLink {
    shared: Weak<Shared>,
}

impl CatalogLink {
    /// Opens a further window on `page` once the event the application is handling is over;
    /// nothing opens once the catalog has closed.
    pub(crate) fn open_window_later(&self, page: Page) {
        if let Some(shared) = self.shared.upgrade() {
            Catalog { shared }.open_window_later(page);
        }
    }

    /// Lets go of the windows the button of their frame closed; nothing once the catalog has
    /// closed.
    fn forget_closed_windows(&self) {
        if let Some(shared) = self.shared.upgrade() {
            shared.windows.borrow_mut().retain(Window::is_open);
        }
    }

    /// Adds a picture to the application of the catalog; none once the catalog has closed.
    fn add_image(&self, encoded: Vec<u8>) -> Option<ImageId> {
        self.shared
            .upgrade()
            .map(|shared| shared.application.add_image(encoded))
    }

    /// Runs `work` with the application of the catalog once the event the application is
    /// handling is over; nothing runs once the catalog has closed.
    fn defer(&self, work: impl FnOnce(&Application) + 'static) {
        if let Some(shared) = self.shared.upgrade() {
            let application = shared.application.clone();
            shared.application.defer(move || work(&application));
        }
    }
}

/// What every handle of the catalog reaches.
#[derive(Debug)]
struct Shared {
    application: Application,
    gallery: Gallery,
    windows: RefCell<Vec<Window>>,
    /// How many windows the catalog has opened, the closed ones among them.
    opened_windows: Cell<usize>,
    appearance: Signal<Appearance>,
    token_set: Signal<TokenSet>,
}

impl Catalog {
    /// Starts the catalog: the application with the styles of the catalog and the painter of
    /// its canvases, and its first window on `first_page`.
    ///
    /// # Panics
    ///
    /// When called from a thread other than the main one.
    pub fn launch(first_page: Page) -> Result<Self, RendererError> {
        let application = Application::new(styles())?;
        application.set_canvas_painter(DiskMap::painter());
        let (appearance, token_set) =
            application.update(|runtime| (runtime.signal(Appearance::System), runtime.signal(TokenSet::Base)));
        let shared = Rc::new_cyclic(|shared| {
            let dropping = CatalogLink { shared: shared.clone() };
            Shared {
                gallery: Gallery::new(
                    |encoded| application.add_image(encoded),
                    move |encoded| dropping.add_image(encoded),
                ),
                application,
                windows: RefCell::new(Vec::new()),
                opened_windows: Cell::new(0),
                appearance,
                token_set,
            }
        });
        let catalog = Self { shared };
        catalog.follow_settings();
        catalog.open_window(first_page);
        Ok(catalog)
    }

    pub fn application(&self) -> &Application {
        &self.shared.application
    }

    /// The pictures the pages of the catalog show.
    pub fn gallery(&self) -> &Gallery {
        &self.shared.gallery
    }

    /// The windows of the catalog, in the order they were opened. A window the button of its
    /// frame closed leaves them on the next turn of the run loop.
    pub fn windows(&self) -> Ref<'_, [Window]> {
        Ref::map(self.shared.windows.borrow(), Vec::as_slice)
    }

    /// The appearance every window shows; the toolbar of a window writes it.
    pub fn appearance(&self) -> Signal<Appearance> {
        self.shared.appearance
    }

    /// The values every window gives the tokens; the toolbar of a window writes it.
    pub fn token_set(&self) -> Signal<TokenSet> {
        self.shared.token_set
    }

    /// Opens a further window of the catalog on `page`.
    ///
    /// # Panics
    ///
    /// When called while the application tells a handler of an event: a handler opens a window
    /// through [`open_window_later`](Self::open_window_later).
    pub fn open_window(&self, page: Page) {
        let number = self.shared.opened_windows.get() + 1;
        self.shared.opened_windows.set(number);
        let title = if number == 1 {
            String::from(WINDOW_TITLE)
        } else {
            format!("{WINDOW_TITLE} {number}")
        };
        let options = WindowOptions {
            title,
            content_size: WINDOW_SIZE,
            blur_behind: false,
        };
        let window = self
            .shared
            .application
            .open_window(&options, |mount| catalog_window::mount(mount, self, page));
        let closing = self.link();
        window.on_closed(move || closing.forget_closed_windows());
        self.shared.windows.borrow_mut().push(window);
    }

    /// What the handlers and the effects of the catalog reach it by.
    pub(crate) fn link(&self) -> CatalogLink {
        CatalogLink {
            shared: Rc::downgrade(&self.shared),
        }
    }

    /// Opens a further window on `page` once the event the application is handling is over.
    pub fn open_window_later(&self, page: Page) {
        let catalog = self.clone();
        self.shared.application.defer(move || catalog.open_window(page));
    }

    /// Runs the event loop until the catalog quits.
    pub fn run(&self) {
        self.shared.application.run();
    }

    /// Shows the appearance and the token set the signals hold in every window, from now on.
    /// The signals are written in the middle of an event, where the application cannot restyle
    /// its windows: the windows follow once the event is over.
    fn follow_settings(&self) {
        let (appearance, token_set) = (self.shared.appearance, self.shared.token_set);
        let restyling = self.link();
        let reappearing = self.link();
        self.shared.application.update(move |runtime| {
            runtime.effect(move |runtime, _| {
                let token_set = runtime.get(token_set);
                restyling.defer(move |application| application.set_themes(token_set.light(), token_set.dark()));
            });
            runtime.effect(move |runtime, _| {
                let appearance = runtime.get(appearance);
                reappearing.defer(move |application| application.set_appearance(appearance));
            });
        });
    }
}
