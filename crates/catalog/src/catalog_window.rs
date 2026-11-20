use std::ops::RangeInclusive;

use morok::Appearance;
use morok::components::view::Mount;
use morok::components::{
    IconButton, KitIcon, SegmentedControl, Sidebar, SidebarItem, SplitPane, SplitView, StatusBar, Toolbar,
};
use morok::core::{ElementId, Key, KeyChord, Modifiers};
use morok::reactive::Signal;

use crate::catalog::Catalog;
use crate::page::Page;
use crate::styles::CLASSES;
use crate::token_set::TokenSet;

/// How wide the sidebar of the pages opens, and how far its divider moves.
const SIDEBAR_WIDTH: f32 = 210.0;
const SIDEBAR_LIMITS: RangeInclusive<f32> = 160.0..=300.0;
/// The chord of the command that opens a further window.
pub const NEW_WINDOW: KeyChord = KeyChord {
    key: Key::Character('n'),
    modifiers: Modifiers::COMMAND,
};

/// What a window of the catalog shows: a toolbar with the appearance and the token set of the
/// catalog, the pages in a sidebar beside the page that is open, and a status bar that says
/// what the page shows. Its root registers the command that opens a further window, in the
/// menu "File" of the menu bar.
pub(crate) fn mount(mount: &mut Mount<'_>, catalog: &Catalog, first_page: Page) -> ElementId {
    let page = mount.runtime().signal(first_page);
    let sidebar_width = mount.runtime().signal(SIDEBAR_WIDTH);
    let appearance = catalog.appearance();
    let token_set = catalog.token_set();
    let opening = catalog.link();
    mount
        .surface_root()
        .menu_command("File", "New Window", NEW_WINDOW, move |runtime| {
            opening.open_window_later(runtime.get(page));
        });

    let opening = catalog.link();
    let toolbar = Toolbar::new()
        .title("MOROK catalog")
        .spacer()
        .item(move |mount| {
            SegmentedControl::new(appearance)
                .segment(Appearance::System, "Auto")
                .segment(Appearance::Light, "Light")
                .segment(Appearance::Dark, "Dark")
                .label("Appearance")
                .mount(mount)
        })
        .item(move |mount| {
            SegmentedControl::new(token_set)
                .segment(TokenSet::Base, TokenSet::Base.title())
                .segment(TokenSet::Ember, TokenSet::Ember.title())
                .label("Tokens")
                .mount(mount)
        })
        .item(move |mount| {
            IconButton::new(KitIcon::Plus, "New window")
                .on_press(move |runtime| opening.open_window_later(runtime.get(page)))
                .mount(mount)
        })
        .label("Catalog")
        .mount(mount);
    let split_view = SplitView::new()
        .pane(SplitPane::sized(sidebar_width, SIDEBAR_LIMITS, move |mount| {
            pages_sidebar(mount, page)
        }))
        .pane(SplitPane::flexible(move |mount| page_room(mount, page)))
        .label("Catalog")
        .mount(mount);
    let body = mount.element().class(CLASSES.catalog_body).child(split_view).id();
    let shown = mount
        .runtime()
        .memo(move |runtime| runtime.get(page).components().to_owned());
    let status_bar = StatusBar::new()
        .text(shown)
        .spacer()
        .text("⌘N — new window")
        .label("Status")
        .mount(mount);
    mount
        .element()
        .class(CLASSES.catalog_window)
        .child(toolbar)
        .child(body)
        .child(status_bar)
        .id()
}

/// The pages of the families and of the screens, the open one selected.
fn pages_sidebar(mount: &mut Mount<'_>, page: Signal<Page>) -> ElementId {
    let mut sidebar = Sidebar::new(page).heading("COMPONENTS");
    for family in Page::FAMILIES {
        sidebar = sidebar.item(SidebarItem::new(family, family.title()).icon(family.icon()));
    }
    sidebar = sidebar.heading("SCREENS");
    for screen in Page::SCREENS {
        sidebar = sidebar.item(SidebarItem::new(screen, screen.title()).icon(screen.icon()));
    }
    sidebar.label("Pages").mount(mount)
}

/// The room of the open page: a page is built when it opens and goes, with its signals, when
/// another opens.
fn page_room(mount: &mut Mount<'_>, page: Signal<Page>) -> ElementId {
    let is_open = Page::FAMILIES
        .into_iter()
        .chain(Page::SCREENS)
        .map(|shown| (shown, mount.runtime().memo(move |runtime| runtime.get(page) == shown)))
        .collect::<Vec<_>>();
    let mut room = mount.element();
    room.class(CLASSES.catalog_page);
    for (shown, is_open) in is_open {
        room.show(is_open, move |mount| shown.mount(mount));
    }
    room.id()
}
