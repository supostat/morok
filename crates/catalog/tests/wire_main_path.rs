mod support;

use std::process::ExitCode;
use std::time::Duration;

use morok::FrameImage;
use morok::core::{ElementId, ElementTree, Point, Rect, Role};
use morok::style::{Color, Theme};
use morok_catalog::styles::CLASSES;
use morok_catalog::tokens::TOKENS;
use morok_catalog::{Catalog, DiskMap, Page, TABLE_ROWS, WINDOW_SIZE, WINDOW_TITLE, ember, tokens};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSMenu, NSWindow};
use support::{AccessibleElement, EventSource, Expectations, KeyStroke, visible_windows, window_titled};

/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// Long enough for the frames an input asks for to be drawn, for a window to open or close
/// and for the work a handler hands on to be done.
const ONE_CHANGE: Duration = Duration::from_millis(700);
/// How far a channel of a pixel the renderer drew may be from the value of its token.
const CHANNEL_TOLERANCE: i32 = 2;
/// How high the table of the kit makes a row.
const TABLE_ROW_HEIGHT: f32 = 26.0;
/// How far the wheel scrolls the table: a thousand rows.
const WHEEL_SCROLL: i32 = 26_000;
/// More elements than a window of the catalog builds while it shows a page of a family.
const ELEMENTS_OF_A_PAGE: usize = 3000;
const SECOND_WINDOW_TITLE: &str = "MOROK catalog 2";
const DOCUMENT: &str = "Document ";

/// The catalog as the test drives it, with what the expectations read of it.
struct Stage {
    main_thread: MainThreadMarker,
    catalog: Catalog,
    expectations: Expectations,
}

impl Stage {
    fn settle(&self, duration: Duration) {
        self.catalog.application().run_for(duration);
    }

    /// Reads the tree of the window opened `opened`-th, counted from zero.
    fn tree_of<Outcome>(&self, opened: usize, read: impl FnOnce(&mut ElementTree) -> Outcome) -> Outcome {
        let windows = self.catalog.windows();
        windows[opened].with_tree(read)
    }

    fn frame_of(&self, opened: usize) -> FrameImage {
        let windows = self.catalog.windows();
        windows[opened].frame_image()
    }

    fn native_window(&self, opened: usize) -> Option<Retained<NSWindow>> {
        let title = if opened == 0 { WINDOW_TITLE } else { SECOND_WINDOW_TITLE };
        window_titled(self.main_thread, title)
    }

    fn events_of(&self, opened: usize) -> Option<EventSource> {
        Some(EventSource::for_window(self.native_window(opened)?, WINDOW_SIZE))
    }

    /// Clicks the first element of the window that shows `text`, and lets the change show.
    /// Returns whether the window shows the text.
    fn click_text(&mut self, opened: usize, text: &str) -> bool {
        let at = self.tree_of(opened, |tree| {
            find_text(tree, text).map(|element| middle(tree.border_box(element)))
        });
        let (Some(at), Some(events)) = (at, self.events_of(opened)) else {
            self.expectations
                .expect(false, "the window shows what the test clicks", text);
            return false;
        };
        events.click(at);
        self.settle(ONE_CHANGE);
        true
    }

    /// Whether the window shows the text in an element of its own.
    fn shows_text(&self, opened: usize, text: &str) -> bool {
        self.tree_of(opened, |tree| find_text(tree, text).is_some())
    }

    /// The accessibility elements of the window, as an assistive technology finds them.
    fn accessible_elements(&self, opened: usize) -> Vec<AccessibleElement> {
        self.events_of(opened).map_or_else(Vec::new, |events| {
            AccessibleElement::of_view(events.surface_view()).descendants()
        })
    }

    /// Whether the system sees an element of the role whose title or value is `text`.
    fn sees(&self, opened: usize, role: &str, text: &str) -> bool {
        self.accessible_elements(opened).iter().any(|element| {
            element.role().as_deref() == Some(role)
                && (element.title().as_deref() == Some(text) || element.value().as_deref() == Some(text))
        })
    }

    /// The point of the room of the page that no page covers: the bottom right of the room.
    fn bare_page_room(&self, opened: usize) -> Point {
        self.tree_of(opened, |tree| {
            let room = elements(tree)
                .into_iter()
                .find(|element| tree.classes(*element).contains(&CLASSES.catalog_page))
                .map(|room| tree.border_box(room))
                .unwrap_or_default();
            Point {
                x: room.x + room.width - 8.0,
                y: room.y + room.height - 8.0,
            }
        })
    }
}

/// The elements of the tree in document order.
fn elements(tree: &ElementTree) -> Vec<ElementId> {
    let mut elements = Vec::new();
    let mut unvisited = vec![tree.root()];
    while let Some(element) = unvisited.pop() {
        elements.push(element);
        unvisited.extend(tree.children(element).iter().rev());
    }
    elements
}

fn find_text(tree: &ElementTree, text: &str) -> Option<ElementId> {
    elements(tree)
        .into_iter()
        .find(|element| tree.role(*element) == Role::Text && tree.text(*element) == text)
}

fn middle(rect: Rect) -> Point {
    Point {
        x: rect.x + rect.width / 2.0,
        y: rect.y + rect.height / 2.0,
    }
}

/// The numbers of the documents whose rows the table shows.
fn shown_documents(tree: &ElementTree) -> Vec<usize> {
    elements(tree)
        .into_iter()
        .filter(|element| tree.role(*element) == Role::Text)
        .filter_map(|element| tree.text(element).strip_prefix(DOCUMENT)?.parse().ok())
        .collect()
}

/// The color of a pixel of the frame at a point of the surface.
fn pixel_at(image: &FrameImage, at: Point) -> [u8; 4] {
    let scale = image.width as f32 / WINDOW_SIZE.width;
    image.pixel((at.x * scale) as usize, (at.y * scale) as usize)
}

/// Whether the pixel shows an opaque color.
fn is_near(pixel: [u8; 4], color: Color) -> bool {
    let expected = [color.red, color.green, color.blue, 255];
    pixel
        .iter()
        .zip(expected)
        .all(|(channel, expected)| (i32::from(*channel) - i32::from(expected)).abs() <= CHANNEL_TOLERANCE)
}

fn background(theme: &Theme) -> Color {
    *theme.resolve(TOKENS.color_bg)
}

fn the_catalog_launches_on_its_first_page(stage: &mut Stage) {
    let windows = visible_windows(stage.main_thread);
    let titles: Vec<String> = windows.iter().map(|window| window.title().to_string()).collect();
    stage.expectations.expect(
        titles == [WINDOW_TITLE],
        "the catalog opens one window",
        &format!("{titles:?}"),
    );
    let presented = stage.catalog.windows()[0].frame_statistics().presented_frames;
    stage.expectations.expect(
        presented > 0,
        "the window presents its first frame",
        &presented.to_string(),
    );
    let missing: Vec<&str> = Page::FAMILIES
        .into_iter()
        .chain(Page::SCREENS)
        .map(Page::title)
        .filter(|title| !stage.shows_text(0, title))
        .collect();
    stage.expectations.expect(
        missing.is_empty(),
        "the sidebar lists every page of a family and every screen",
        &format!("missing: {missing:?}"),
    );
    stage.expectations.expect(
        stage.sees(0, "AXButton", "Reveal") && stage.sees(0, "AXButton", "Clean up 4.2 GB"),
        "the system sees the buttons of the first page through the accessibility tree",
        "no button named Reveal and Clean up 4.2 GB",
    );
}

fn the_light_appearance_draws_the_light_values_of_the_tokens(stage: &mut Stage) {
    stage.click_text(0, "Light");
    let room = stage.bare_page_room(0);
    let pixel = pixel_at(&stage.frame_of(0), room);
    let is_light = stage.tree_of(0, |tree| *tree.theme() == tokens::LIGHT);
    stage.expectations.expect(
        is_light && is_near(pixel, background(&tokens::LIGHT)),
        "the light appearance styles the window with the light values and the frame shows them",
        &format!("light theme: {is_light}, the room of the page is {pixel:?}"),
    );
}

fn a_click_presses_a_button(stage: &mut Stage) {
    stage.click_text(0, "Reveal");
    stage.expectations.expect(
        stage.shows_text(0, "pressed 1 times"),
        "a click on a button runs its handler, and what the handler wrote shows",
        "the page does not say it was pressed once",
    );
}

fn typing_fills_a_text_field(stage: &mut Stage) {
    stage.click_text(0, "Text fields");
    let field = stage.tree_of(0, |tree| {
        elements(tree)
            .into_iter()
            .find(|element| tree.role(*element) == Role::TextInput && tree.placeholder(*element) == "Name")
            .map(|field| middle(tree.border_box(field)))
    });
    let (Some(field), Some(events)) = (field, stage.events_of(0)) else {
        stage.expectations.expect(
            false,
            "the page of the text fields shows the field of the name",
            "no such field",
        );
        return;
    };
    events.click(field);
    stage.settle(ONE_CHANGE);
    events.type_letters("quartz");
    stage.settle(ONE_CHANGE);
    stage.expectations.expect(
        stage.shows_text(0, "name: \"quartz\""),
        "the keys typed go through the input method of the system into the focused field",
        "the page does not read back the name typed",
    );
    stage.expectations.expect(
        stage.sees(0, "AXTextField", "quartz"),
        "the system sees the text the field holds",
        "no text field holds quartz",
    );
}

fn the_wheel_and_the_keys_scroll_a_million_rows(stage: &mut Stage) {
    stage.click_text(0, "Table");
    let before = stage.tree_of(0, |tree| shown_documents(tree));
    let table = stage.tree_of(0, |tree| {
        elements(tree)
            .into_iter()
            .find(|element| tree.classes(*element).contains(&CLASSES.table))
            .map(|table| middle(tree.border_box(table)))
    });
    let (Some(table), Some(events)) = (table, stage.events_of(0)) else {
        stage
            .expectations
            .expect(false, "the page of the table shows a table", "no table");
        return;
    };
    stage.expectations.expect(
        before.iter().min() == Some(&1),
        "the table opens on its first row",
        &format!("{before:?}"),
    );

    events.scroll_down(stage.main_thread, table, WHEEL_SCROLL);
    stage.settle(ONE_CHANGE);
    let after_wheel = stage.tree_of(0, |tree| shown_documents(tree));
    let first_after_wheel = after_wheel.iter().min().copied().unwrap_or_default();
    let rows_scrolled = (WHEEL_SCROLL as f32 / TABLE_ROW_HEIGHT) as usize;
    stage.expectations.expect(
        first_after_wheel.abs_diff(rows_scrolled) <= 2,
        "the wheel scrolls the table by the rows its pixels cover",
        &format!("the first row shown is {first_after_wheel}, {rows_scrolled} rows were scrolled"),
    );

    events.click(table);
    stage.settle(ONE_CHANGE);
    events.press(KeyStroke::END);
    stage.settle(ONE_CHANGE);
    let at_end = stage.tree_of(0, |tree| shown_documents(tree));
    let elements_built = stage.tree_of(0, |tree| elements(tree).len());
    stage.expectations.expect(
        at_end.contains(&TABLE_ROWS),
        "End takes the keyboard to the last of a million rows and scrolls it into view",
        &format!("{:?}", at_end.iter().max()),
    );
    stage.expectations.expect(
        elements_built < ELEMENTS_OF_A_PAGE,
        "a million rows are shown by as many elements as the viewport needs",
        &format!("{elements_built} elements"),
    );
}

/// The overlay window over the window of the catalog, while one is open.
fn overlay_window(stage: &Stage) -> Option<Retained<NSWindow>> {
    visible_windows(stage.main_thread)
        .into_iter()
        .find(|window| window.title().to_string() != WINDOW_TITLE)
}

fn a_popover_and_a_menu_open_over_the_window(stage: &mut Stage) {
    stage.click_text(0, "Overlays");
    stage.click_text(0, "Show popover");
    let popover = overlay_window(stage);
    let seen: Vec<(Option<String>, Option<String>)> = popover.as_ref().map_or_else(Vec::new, |popover| {
        let view = popover.contentView().expect("an overlay window shows its surface");
        AccessibleElement::of_view(view)
            .descendants()
            .iter()
            .map(|element| (element.role(), element.title()))
            .collect()
    });
    let is_shown = |role: &str, title: &str| seen.contains(&(Some(role.to_owned()), Some(title.to_owned())));
    stage.expectations.expect(
        is_shown("AXWindow", "Xcode cache") && is_shown("AXButton", "Clean up") && is_shown("AXButton", "Later"),
        "the popover opens in a window of its own, which the system sees with its name and its buttons",
        &format!("{seen:?}"),
    );
    if let Some(popover) = popover {
        EventSource::for_window(popover, WINDOW_SIZE).press(KeyStroke::ESCAPE);
        stage.settle(ONE_CHANGE);
    }
    stage.expectations.expect(
        overlay_window(stage).is_none(),
        "Escape closes the popover and its window",
        "an overlay window is still on screen",
    );

    stage.click_text(0, "Show menu");
    let menu = overlay_window(stage);
    stage.expectations.expect(
        menu.is_some(),
        "the menu opens in a window of its own",
        "no overlay window",
    );
    if let Some(menu) = menu {
        let keys = EventSource::for_window(menu, WINDOW_SIZE);
        keys.press(KeyStroke::DOWN);
        stage.settle(ONE_CHANGE);
        keys.press(KeyStroke::RETURN);
        stage.settle(ONE_CHANGE);
    }
    stage.expectations.expect(
        stage.shows_text(0, "last answer: quick look") && overlay_window(stage).is_none(),
        "the arrow and Return choose the first line of the menu, which closes",
        "the page does not read back the line chosen",
    );
}

fn the_theme_and_the_tokens_switch(stage: &mut Stage) {
    stage.click_text(0, "Dark");
    let room = stage.bare_page_room(0);
    let dark_pixel = pixel_at(&stage.frame_of(0), room);
    let is_dark = stage.tree_of(0, |tree| *tree.theme() == tokens::DARK);
    stage.expectations.expect(
        is_dark && is_near(dark_pixel, background(&tokens::DARK)),
        "the dark appearance restyles the window with the dark values and the frame shows them",
        &format!("dark theme: {is_dark}, the room of the page is {dark_pixel:?}"),
    );

    stage.click_text(0, "Ember");
    let ember_pixel = pixel_at(&stage.frame_of(0), room);
    let is_ember = stage.tree_of(0, |tree| *tree.theme() == ember::DARK);
    stage.expectations.expect(
        is_ember && is_near(ember_pixel, background(&ember::DARK)),
        "the ember tokens restyle the window with their dark values and the frame shows them",
        &format!("ember theme: {is_ember}, the room of the page is {ember_pixel:?}"),
    );
}

/// The menu "File" of the menu bar.
fn file_menu(main_thread: MainThreadMarker) -> Option<Retained<NSMenu>> {
    let bar = NSApplication::sharedApplication(main_thread).mainMenu()?;
    (0..bar.numberOfItems())
        .filter_map(|position| bar.itemAtIndex(position))
        .find(|item| item.title().to_string() == "File")
        .and_then(|item| item.submenu())
}

fn a_second_window_opens_from_the_menu_bar(stage: &mut Stage) {
    let Some(menu) = file_menu(stage.main_thread) else {
        stage
            .expectations
            .expect(false, "the menu bar holds the menu File of the catalog", "no such menu");
        return;
    };
    menu.performActionForItemAtIndex(0);
    stage.settle(SETTLING);
    let window_count = stage.catalog.windows().len();
    stage.expectations.expect(
        window_count == 2 && stage.native_window(1).is_some(),
        "New Window of the menu bar opens a second window of the catalog",
        &format!("{window_count} windows"),
    );
    if window_count < 2 {
        return;
    }
    let second_is_ember = stage.tree_of(1, |tree| *tree.theme() == ember::DARK);
    let shows_the_same_page = stage.tree_of(1, |tree| {
        elements(tree)
            .into_iter()
            .filter(|element| tree.role(*element) == Role::Text && tree.text(*element) == "Overlays")
            .count()
            == 2
    });
    stage.expectations.expect(
        second_is_ember && shows_the_same_page,
        "the second window opens on the page of the first, in the appearance and the tokens shown",
        &format!("ember: {second_is_ember}, the same page: {shows_the_same_page}"),
    );

    stage.click_text(1, "Disk map");
    let focus_of_first = stage.tree_of(0, |tree| tree.focused());
    let focus_of_second = stage.tree_of(1, |tree| tree.focused());
    stage.expectations.expect(
        focus_of_first != focus_of_second || focus_of_first.is_none(),
        "the two windows keep focus of their own",
        &format!("{focus_of_first:?}, {focus_of_second:?}"),
    );
    let first_folder = DiskMap::sunburst().sectors()[0];
    let ring = first_folder.sector;
    let at = stage.tree_of(1, |tree| {
        elements(tree)
            .into_iter()
            .find(|element| tree.role(*element) == Role::Canvas)
            .map(|canvas| {
                let canvas = tree.border_box(canvas);
                let angle = f32::midpoint(ring.start_angle, ring.end_angle);
                let radius = f32::midpoint(ring.inner_radius, ring.outer_radius);
                Point {
                    x: canvas.x + ring.center.x + radius * angle.cos(),
                    y: canvas.y + ring.center.y + radius * angle.sin(),
                }
            })
    });
    let pixel = at.map(|at| pixel_at(&stage.frame_of(1), at));
    let chart_color = *ember::DARK.resolve(first_folder.color);
    stage.expectations.expect(
        pixel.is_some_and(|pixel| is_near(pixel, chart_color)),
        "the painter of the catalog draws the sunburst into the canvas in the colors of the tokens shown",
        &format!("{pixel:?}, the token is {chart_color:?}"),
    );
    let describes_chart = stage.accessible_elements(1).iter().any(|element| {
        element.role().as_deref() == Some("AXImage")
            && element
                .title()
                .or_else(|| element.value())
                .is_some_and(|title| title.starts_with("Macintosh HD, 612 GB used of 1 TB"))
    });
    stage.expectations.expect(
        describes_chart,
        "the system sees the canvas as an image that tells what it shows",
        "no image tells of the folders of Macintosh HD",
    );
}

fn main() -> ExitCode {
    let main_thread = MainThreadMarker::new().expect("a test that opens windows runs on the main thread");
    let catalog = Catalog::launch(Page::Buttons).expect("this Mac runs Metal 4");
    let mut stage = Stage {
        main_thread,
        catalog,
        expectations: Expectations::default(),
    };
    stage.settle(SETTLING);

    the_catalog_launches_on_its_first_page(&mut stage);
    the_light_appearance_draws_the_light_values_of_the_tokens(&mut stage);
    a_click_presses_a_button(&mut stage);
    typing_fills_a_text_field(&mut stage);
    the_wheel_and_the_keys_scroll_a_million_rows(&mut stage);
    a_popover_and_a_menu_open_over_the_window(&mut stage);
    the_theme_and_the_tokens_switch(&mut stage);
    a_second_window_opens_from_the_menu_bar(&mut stage);

    stage.expectations.report("wire main path")
}
