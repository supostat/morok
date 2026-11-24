mod support;

use std::io::Cursor;
use std::process::ExitCode;
use std::time::Duration;

use morok::core::{ElementId, ElementTree, ImageId, Point, Rect, Role, Size};
use morok::style::{Class, Color, States, Theme};
use morok::{FrameImage, RingSector};
use morok_catalog::styles::CLASSES;
use morok_catalog::tokens::TOKENS;
use morok_catalog::{
    CHART_SIZE, Catalog, ChartView, GalleryPicture, Page, TABLE_ROWS, Volume, WINDOW_SIZE, WINDOW_TITLE, ember, tokens,
};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSAccessibilityOrientation, NSApplication, NSFloatingWindowLevel, NSMenu, NSNormalWindowLevel, NSScreen, NSWindow,
};
use objc2_foundation::NSPoint;
use support::{
    AccessibleElement, EventSource, Expectations, KeyStroke, bring_to_front, visible_windows, window_titled,
};

/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// Long enough for the frames an input asks for to be drawn, for a window to open or close
/// and for the work a handler hands on to be done.
const ONE_CHANGE: Duration = Duration::from_millis(700);
/// How far a channel of a pixel the renderer drew may be from the value of its token.
const CHANNEL_TOLERANCE: i32 = 2;
/// How far a channel of a pixel of a picture the renderer drew may be from the pixel of its
/// file: the picture is scaled to the screen and filtered on the way.
const PICTURE_TOLERANCE: i32 = 8;
/// How high the table of the kit makes a row.
const TABLE_ROW_HEIGHT: f32 = 26.0;
/// How far the wheel scrolls the table: a thousand rows.
const WHEEL_SCROLL: i32 = 26_000;
/// The most pixels the test turns the wheel by in one event.
const LONGEST_WHEEL_TURN: i32 = 26_000;
/// 2^24 + 1 pixels: an offset single precision rounds to 2^24.
const ODD_FAR_DOWN: i32 = 16_777_217;
/// More elements than a window of the catalog builds while it shows a page of a family.
const ELEMENTS_OF_A_PAGE: usize = 3000;
const SECOND_WINDOW_TITLE: &str = "MOROK catalog 2";
const THIRD_WINDOW_TITLE: &str = "MOROK catalog 3";
const DOCUMENT: &str = "Document ";
/// The hosts the combobox of the pickers page offers, in their order.
const HOSTS: [&str; 4] = ["quartz.local", "quartz-backup.local", "domovoi.local", "mneme.local"];
/// What the user types into the combobox of hosts: two of them hold it.
const TYPED_HOST: &str = "quartz";
const MEADOW_PNG: &[u8] = include_bytes!("../images/meadow.png");
const MEADOW_SIZE: Size = Size {
    width: 160.0,
    height: 100.0,
};
/// Pixels of the meadow, counted from its top left: the sky, the sun and the grass.
const MEADOW_PROBES: [(usize, usize); 3] = [(10, 10), (125, 27), (80, 95)];
/// How far right of the first window the second one stands: it covers the right half of the
/// first and leaves the pictures of its pages bare.
const SECOND_WINDOW_SHIFT: f64 = 560.0;
/// How far inside the rectangle of a folder of the treemap the test reads the band that shows
/// the folder itself.
const INTO_FOLDER_BAND: f32 = 2.0;
/// How much room the test leaves around an element it scrolls into view.
const VIEW_MARGIN: f32 = 16.0;
/// How far apart two edges of boxes the layout lines up may lie.
const LAYOUT_TOLERANCE: f32 = 0.5;

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

    /// The middle of the first element of the window that shows `text`, as the surface shows
    /// it.
    fn middle_of_text(&self, opened: usize, text: &str) -> Option<Point> {
        self.tree_of(opened, |tree| {
            find_text(tree, text).map(|element| middle(tree.surface_box(element)))
        })
    }

    /// Clicks the first element of the window that shows `text`, and lets the change show.
    /// Returns whether the window shows the text.
    fn click_text(&mut self, opened: usize, text: &str) -> bool {
        let (Some(at), Some(events)) = (self.middle_of_text(opened, text), self.events_of(opened)) else {
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

    /// Turns the wheel over the scroll view around the element until the whole element lies
    /// inside it, and lets the frames that follow show it.
    fn scroll_into_view(&self, opened: usize, element: ElementId) {
        let placed = self.tree_of(opened, |tree| {
            let scroller = scroller_around(tree, element)?;
            Some((tree.surface_box(scroller), tree.surface_box(element)))
        });
        let (Some((viewport, shown)), Some(events)) = (placed, self.events_of(opened)) else {
            return;
        };
        let past_bottom = shown.bottom() + VIEW_MARGIN - viewport.bottom();
        let past_top = viewport.y - (shown.y - VIEW_MARGIN);
        let distance = if past_bottom > 0.0 {
            past_bottom
        } else if past_top > 0.0 {
            -past_top
        } else {
            return;
        };
        events.scroll_down(self.main_thread, middle(viewport), distance.ceil() as i32);
        self.settle(ONE_CHANGE);
    }

    /// Whether the whole element lies inside the scroll view around it, where the window
    /// shows it.
    fn is_in_view(&self, opened: usize, element: ElementId) -> bool {
        self.tree_of(opened, |tree| {
            scroller_around(tree, element).is_some_and(|scroller| {
                let (viewport, shown) = (tree.surface_box(scroller), tree.surface_box(element));
                shown.y >= viewport.y && shown.bottom() <= viewport.bottom()
            })
        })
    }
}

/// The elements of the tree in document order.
fn elements(tree: &ElementTree) -> Vec<ElementId> {
    descendants(tree, tree.root())
}

/// The element and the elements under it, in document order.
fn descendants(tree: &ElementTree, element: ElementId) -> Vec<ElementId> {
    let mut found = Vec::new();
    let mut unvisited = vec![element];
    while let Some(element) = unvisited.pop() {
        found.push(element);
        unvisited.extend(tree.children(element).iter().rev());
    }
    found
}

/// The nearest element around the element that scrolls what it holds.
fn scroller_around(tree: &ElementTree, element: ElementId) -> Option<ElementId> {
    let mut around = tree.parent(element);
    while let Some(ancestor) = around {
        if tree.role(ancestor) == Role::Scroll {
            return Some(ancestor);
        }
        around = tree.parent(ancestor);
    }
    None
}

fn find_text(tree: &ElementTree, text: &str) -> Option<ElementId> {
    elements(tree)
        .into_iter()
        .find(|element| tree.role(*element) == Role::Text && tree.text(*element) == text)
}

/// The first element of the tree that carries every class.
fn find_classed(tree: &ElementTree, classes: &[Class]) -> Option<ElementId> {
    elements(tree).into_iter().find(|element| {
        let carried = tree.classes(*element);
        classes.iter().all(|class| carried.contains(class))
    })
}

fn middle(rect: Rect) -> Point {
    Point {
        x: rect.x + rect.width / 2.0,
        y: rect.y + rect.height / 2.0,
    }
}

/// A point far enough from `point` for a press there to become a drag.
fn beside(point: Point) -> Point {
    Point {
        x: point.x + 12.0,
        ..point
    }
}

/// The numbers of the documents whose rows the table shows.
fn shown_documents(tree: &ElementTree) -> Vec<usize> {
    document_tops(tree).into_iter().map(|(document, _)| document).collect()
}

/// The documents whose rows the table holds, each with where the top of its name lies on the
/// surface.
fn document_tops(tree: &ElementTree) -> Vec<(usize, f32)> {
    elements(tree)
        .into_iter()
        .filter(|element| tree.role(*element) == Role::Text)
        .filter_map(|element| {
            let document = tree.text(element).strip_prefix(DOCUMENT)?.parse().ok()?;
            Some((document, tree.surface_box(element).y))
        })
        .collect()
}

/// The color of a pixel of the frame at a point of the surface.
fn pixel_at(image: &FrameImage, at: Point) -> [u8; 4] {
    let scale = image.width as f32 / WINDOW_SIZE.width;
    image.pixel((at.x * scale) as usize, (at.y * scale) as usize)
}

/// Whether the pixel shows an opaque color.
fn is_near(pixel: [u8; 4], color: Color) -> bool {
    is_within([color.red, color.green, color.blue, 255], pixel, CHANNEL_TOLERANCE)
}

fn is_within(expected: [u8; 4], pixel: [u8; 4], tolerance: i32) -> bool {
    pixel
        .iter()
        .zip(expected)
        .all(|(channel, expected)| (i32::from(*channel) - i32::from(expected)).abs() <= tolerance)
}

fn background(theme: &Theme) -> Color {
    *theme.resolve(TOKENS.color_bg)
}

/// The meadow of the catalog as its file holds it.
fn meadow_as_filed() -> FrameImage {
    let mut reader = png::Decoder::new(Cursor::new(MEADOW_PNG))
        .read_info()
        .expect("the meadow of the catalog is a PNG");
    let mut decoded = vec![0; reader.output_buffer_size().expect("the meadow fits in memory")];
    let frame = reader.next_frame(&mut decoded).expect("the meadow decodes");
    assert_eq!(
        (frame.color_type, frame.bit_depth),
        (png::ColorType::Rgb, png::BitDepth::Eight),
        "the meadow holds eight-bit RGB"
    );
    FrameImage {
        width: frame.width as usize,
        height: frame.height as usize,
        pixels: decoded[..frame.buffer_size()]
            .chunks_exact(3)
            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], u8::MAX])
            .collect(),
    }
}

/// The point in the middle of a piece of a sunburst drawn into the canvas of the box.
fn middle_of_piece(canvas: Rect, piece: RingSector) -> Point {
    let angle = f32::midpoint(piece.start_angle, piece.end_angle);
    let radius = f32::midpoint(piece.inner_radius, piece.outer_radius);
    Point {
        x: canvas.x + piece.center.x + radius * angle.cos(),
        y: canvas.y + piece.center.y + radius * angle.sin(),
    }
}

/// The box of the canvas of the disk map the window shows.
fn chart_canvas(stage: &Stage, opened: usize) -> Option<Rect> {
    stage.tree_of(opened, |tree| {
        elements(tree)
            .into_iter()
            .find(|element| tree.role(*element) == Role::Canvas)
            .map(|canvas| tree.surface_box(canvas))
    })
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
            .map(|field| middle(tree.surface_box(field)))
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

/// The middle of the table of the window, and the element that scrolls its rows.
fn table_of(stage: &Stage) -> Option<(Point, ElementId)> {
    stage.tree_of(0, |tree| {
        let table = find_classed(tree, &[CLASSES.table])?;
        let scroller = descendants(tree, table)
            .into_iter()
            .find(|element| tree.role(*element) == Role::Scroll)?;
        Some((middle(tree.surface_box(table)), scroller))
    })
}

fn the_wheel_and_the_keys_scroll_a_million_rows(stage: &mut Stage) {
    stage.click_text(0, "Table");
    let before = stage.tree_of(0, |tree| shown_documents(tree));
    let (Some((table, _)), Some(events)) = (table_of(stage), stage.events_of(0)) else {
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

fn the_wheel_deep_in_the_rows_moves_them_by_one_pixel(stage: &mut Stage) {
    let (Some((table, scroller)), Some(events)) = (table_of(stage), stage.events_of(0)) else {
        return;
    };
    events.press(KeyStroke::HOME);
    stage.settle(ONE_CHANGE);
    let mut left = ODD_FAR_DOWN;
    while left > 0 {
        let turn = left.min(LONGEST_WHEEL_TURN);
        events.scroll_down(stage.main_thread, table, turn);
        left -= turn;
    }
    stage.settle(ONE_CHANGE);
    let offset = stage.tree_of(0, |tree| tree.scroll_offset(scroller).y);
    stage.expectations.expect(
        offset.to_bits() == f64::from(ODD_FAR_DOWN).to_bits(),
        "the wheel takes the table to an offset past 2^24 that single precision does not hold",
        &format!("the table is scrolled by {offset}"),
    );

    for _ in 0..2 {
        let before = stage.tree_of(0, |tree| document_tops(tree));
        events.scroll_down(stage.main_thread, table, 1);
        stage.settle(ONE_CHANGE);
        let after = stage.tree_of(0, |tree| document_tops(tree));
        let moved: Vec<(usize, f32)> = after
            .iter()
            .filter_map(|(document, top)| {
                let (_, top_before) = before.iter().find(|(shown, _)| shown == document)?;
                Some((*document, top_before - top))
            })
            .collect();
        stage.expectations.expect(
            moved.len() >= 10 && moved.iter().all(|(_, by)| by.to_bits() == 1.0_f32.to_bits()),
            "far down the million rows a turn of the wheel by one pixel moves every row by one pixel",
            &format!("the rows moved up by {moved:?}"),
        );
    }
}

/// The names of the options the list of the combobox shows in its overlay window, in the
/// order of their names; none while no overlay is on screen.
fn options_on_screen(stage: &Stage) -> Vec<String> {
    let Some(overlay) = overlay_window(stage) else {
        return Vec::new();
    };
    let view = overlay.contentView().expect("an overlay window shows its surface");
    let mut names: Vec<String> = AccessibleElement::of_view(view)
        .descendants()
        .iter()
        .filter(|element| element.role().as_deref() == Some("AXStaticText"))
        .filter_map(AccessibleElement::value)
        .collect();
    names.sort();
    names.dedup();
    names
}

fn sorted(names: &[&str]) -> Vec<String> {
    let mut sorted: Vec<String> = names.iter().map(|name| String::from(*name)).collect();
    sorted.sort();
    sorted
}

fn the_chevron_shows_every_option_and_the_field_names_the_highlighted_one(stage: &mut Stage) {
    stage.click_text(0, Page::Pickers.title());
    let parts = stage.tree_of(0, |tree| {
        let field = elements(tree)
            .into_iter()
            .find(|element| tree.role(*element) == Role::TextInput && tree.placeholder(*element) == "Host")?;
        let chevron = find_classed(tree, &[CLASSES.combobox_chevron])?;
        Some((middle(tree.surface_box(field)), middle(tree.surface_box(chevron))))
    });
    let (Some((field, chevron)), Some(events)) = (parts, stage.events_of(0)) else {
        stage.expectations.expect(
            false,
            "the page of the pickers shows the combobox of the hosts",
            "no such combobox",
        );
        return;
    };
    events.click(field);
    stage.settle(ONE_CHANGE);
    events.type_letters(TYPED_HOST);
    stage.settle(ONE_CHANGE);
    let narrowed = options_on_screen(stage);
    stage.expectations.expect(
        narrowed == sorted(&HOSTS[..2]),
        "the text typed narrows the list of the combobox to the hosts that hold it",
        &format!("{narrowed:?}"),
    );

    events.click(chevron);
    stage.settle(ONE_CHANGE);
    events.click(chevron);
    stage.settle(ONE_CHANGE);
    let every = options_on_screen(stage);
    stage.expectations.expect(
        every == sorted(&HOSTS),
        "the chevron closes the list and opens it again with every option, whatever the text",
        &format!("{every:?}"),
    );

    for (place, host) in HOSTS[..2].iter().enumerate() {
        events.press(KeyStroke::DOWN);
        stage.settle(ONE_CHANGE);
        let spoken = format!("{host}, {} of {}", place + 1, HOSTS.len());
        stage.expectations.expect(
            stage.sees(0, "AXGroup", &spoken),
            "the tree of the window, where the keyboard stays, names the option the arrow highlights in the list of another tree",
            &format!("nothing says {spoken:?}"),
        );
    }
    events.press(KeyStroke::ESCAPE);
    stage.settle(ONE_CHANGE);
    stage.expectations.expect(
        overlay_window(stage).is_none(),
        "Escape closes the list of the combobox",
        "an overlay window is still on screen",
    );
}

/// The image of the page that shows the meadow at its natural size.
fn natural_picture(tree: &ElementTree) -> Option<ElementId> {
    let caption = find_text(tree, "natural size")?;
    let cell = tree.parent(caption)?;
    let frame = *tree.children(cell).first()?;
    tree.children(frame).first().copied()
}

fn the_images_page_shows_a_decoded_picture(stage: &mut Stage) {
    stage.click_text(0, Page::Images.title());
    let Some(picture) = stage.tree_of(0, |tree| natural_picture(tree)) else {
        stage.expectations.expect(
            false,
            "the page of the images shows the meadow at its natural size",
            "no such picture",
        );
        return;
    };
    stage.scroll_into_view(0, picture);
    let meadow = stage.catalog.gallery().image(GalleryPicture::Meadow);
    let (shown, laid_out, placed) = stage.tree_of(0, |tree| {
        (
            tree.image(picture),
            tree.border_box(picture).size(),
            tree.surface_box(picture),
        )
    });
    stage.expectations.expect(
        shown == Some(meadow) && laid_out == MEADOW_SIZE && stage.is_in_view(0, picture),
        "the image shows the decoded meadow at its natural size, inside the page",
        &format!("{shown:?} of {laid_out:?} at {placed:?}"),
    );

    let frame = stage.frame_of(0);
    let filed = meadow_as_filed();
    let differing: Vec<String> = MEADOW_PROBES
        .iter()
        .filter_map(|(x, y)| {
            let at = Point {
                x: placed.x + *x as f32 + 0.5,
                y: placed.y + *y as f32 + 0.5,
            };
            let (drawn, held) = (pixel_at(&frame, at), filed.pixel(*x, *y));
            (!is_within(held, drawn, PICTURE_TOLERANCE)).then(|| format!("({x}, {y}): {drawn:?}, the file {held:?}"))
        })
        .collect();
    stage.expectations.expect(
        differing.is_empty(),
        "the frame of the window holds the pixels of the PNG where the picture lies: the sky, the sun and the grass",
        &differing.join("; "),
    );
}

/// The overlay window over the window of the catalog, while one is open.
fn overlay_window(stage: &Stage) -> Option<Retained<NSWindow>> {
    visible_windows(stage.main_thread)
        .into_iter()
        .find(|window| !window.title().to_string().starts_with(WINDOW_TITLE))
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

/// The titles of the menus of the menu bar after the menu of the application.
fn menus_of_bar(main_thread: MainThreadMarker) -> Vec<String> {
    let Some(bar) = NSApplication::sharedApplication(main_thread).mainMenu() else {
        return Vec::new();
    };
    (1..bar.numberOfItems())
        .filter_map(|position| bar.itemAtIndex(position))
        .map(|item| item.title().to_string())
        .collect()
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

    stage.click_text(1, Page::DiskMap.title());
    let focus_of_first = stage.tree_of(0, |tree| tree.focused());
    let focus_of_second = stage.tree_of(1, |tree| tree.focused());
    stage.expectations.expect(
        focus_of_first != focus_of_second || focus_of_first.is_none(),
        "the two windows keep focus of their own",
        &format!("{focus_of_first:?}, {focus_of_second:?}"),
    );
    let first_folder = Volume::Macintosh.sunburst().sectors()[0];
    let pixel =
        chart_canvas(stage, 1).map(|canvas| pixel_at(&stage.frame_of(1), middle_of_piece(canvas, first_folder.sector)));
    let chart_color = *ember::DARK.resolve(first_folder.color);
    stage.expectations.expect(
        pixel.is_some_and(|pixel| is_near(pixel, chart_color)),
        "the painter of the catalog draws the sunburst into the canvas in the colors of the tokens shown",
        &format!("{pixel:?}, the token is {chart_color:?}"),
    );
    stage.expectations.expect(
        stage.sees(1, "AXImage", &Volume::Macintosh.description()),
        "the system sees the canvas as an image that tells what it shows",
        "no image tells of the folders of Macintosh HD",
    );
}

fn choosing_a_volume_and_a_view_redraws_the_disk_map(stage: &mut Stage) {
    let Some(canvas) = chart_canvas(stage, 1) else {
        return;
    };
    let first_folder = Volume::Archive.sunburst().sectors()[0];
    let in_first_folder = middle_of_piece(canvas, first_folder.sector);
    let chart_color = *ember::DARK.resolve(first_folder.color);
    let before = pixel_at(&stage.frame_of(1), in_first_folder);
    stage.click_text(1, Volume::Archive.name());
    let after = pixel_at(&stage.frame_of(1), in_first_folder);
    stage.expectations.expect(
        !is_near(before, chart_color) && is_near(after, chart_color),
        "choosing Archive in the sidebar redraws the canvas with the sunburst of Archive",
        &format!("the pixel was {before:?} and is {after:?}, the first folder of Archive is {chart_color:?}"),
    );
    stage.expectations.expect(
        stage.sees(1, "AXImage", &Volume::Archive.description()),
        "the system hears the canvas tell of the folders of Archive",
        "no image tells of the folders of Archive",
    );

    stage.click_text(1, ChartView::Treemap.title());
    let frame = stage.frame_of(1);
    let treemap = Volume::Archive.treemap();
    let differing: Vec<String> = Volume::Archive
        .folders()
        .iter()
        .zip(treemap.folder_areas())
        .filter_map(|(folder, area)| {
            let band = Point {
                x: canvas.x + area.x + area.width / 2.0,
                y: canvas.y + area.y + INTO_FOLDER_BAND,
            };
            let (drawn, color) = (pixel_at(&frame, band), *ember::DARK.resolve(folder.color));
            (!is_near(drawn, color)).then(|| format!("{}: {drawn:?}, the token is {color:?}", folder.name))
        })
        .collect();
    stage.expectations.expect(
        canvas.size().width.to_bits() == CHART_SIZE.to_bits() && differing.is_empty(),
        "Treemap draws the folders of Archive as rectangles, the band along the top of each in its color",
        &differing.join("; "),
    );
}

/// Places the first window at the left of the screen and the second over the right half of
/// it, both floating above the windows of the other applications, so that the pointer goes
/// from the pages of the first to those of the second whatever the user brings to the front
/// meanwhile.
fn place_second_window_over_the_first(stage: &Stage) {
    let (Some(first), Some(second), Some(screen)) = (
        stage.native_window(0),
        stage.native_window(1),
        NSScreen::mainScreen(stage.main_thread),
    ) else {
        return;
    };
    let room = screen.visibleFrame();
    first.setFrameOrigin(room.origin);
    second.setFrameOrigin(NSPoint::new(room.origin.x + SECOND_WINDOW_SHIFT, room.origin.y));
    for window in [&first, &second] {
        window.setLevel(NSFloatingWindowLevel);
        window.orderFrontRegardless();
    }
    second.makeKeyWindow();
}

/// Lets the windows of the catalog sink back among those of the other applications.
fn let_windows_sink(stage: &Stage) {
    for window in [stage.native_window(0), stage.native_window(1)].into_iter().flatten() {
        window.setLevel(NSNormalWindowLevel);
    }
}

/// Which window the pointer would hit at a point of the surface of `events`: that window, or
/// the number of another one in front of it.
fn window_hit(stage: &Stage, events: &EventSource, at: Point) -> String {
    let hit = NSWindow::windowNumberAtPoint_belowWindowWithWindowNumber(events.on_screen(at), 0, stage.main_thread);
    if hit == events.window_number() {
        String::from("the window itself")
    } else {
        format!("window {hit} in front of it")
    }
}

/// The frame of the first picture of the page that shows the picture.
fn frame_of_picture(tree: &ElementTree, image: ImageId) -> Option<Rect> {
    let picture = elements(tree)
        .into_iter()
        .find(|element| tree.role(*element) == Role::Image && tree.image(*element) == Some(image))?;
    tree.parent(picture).map(|frame| tree.surface_box(frame))
}

/// The picture the drop zone of the window shows, if it shows one.
fn picture_in_zone(tree: &ElementTree, zone: ElementId) -> Option<ImageId> {
    tree.children(zone)
        .iter()
        .find(|element| tree.role(**element) == Role::Image)
        .and_then(|picture| tree.image(*picture))
}

fn a_picture_goes_by_the_pointer_to_the_drop_zone_of_the_other_window(stage: &mut Stage) {
    stage.click_text(0, Page::Images.title());
    stage.click_text(1, Page::Images.title());
    place_second_window_over_the_first(stage);
    stage.settle(ONE_CHANGE);
    drag_pictures_from_the_first_window_to_the_second(stage);
    let_windows_sink(stage);
}

fn drag_pictures_from_the_first_window_to_the_second(stage: &mut Stage) {
    let Some(zone) = stage.tree_of(1, |tree| find_classed(tree, &[CLASSES.page_drop_zone])) else {
        stage.expectations.expect(
            false,
            "the page of the images of the second window shows its drop zone",
            "no drop zone",
        );
        return;
    };
    stage.scroll_into_view(1, zone);
    let gallery = stage.catalog.gallery().clone();
    let pictures = stage.tree_of(0, |tree| {
        Some((
            middle(frame_of_picture(tree, gallery.image(GalleryPicture::Dunes))?),
            middle(frame_of_picture(tree, gallery.image(GalleryPicture::Meadow))?),
        ))
    });
    let zone_middle = stage.tree_of(1, |tree| middle(tree.surface_box(zone)));
    let (Some((on_dunes, on_meadow)), Some(source), Some(target)) = (pictures, stage.events_of(0), stage.events_of(1))
    else {
        stage.expectations.expect(
            false,
            "the page of the images of the first window shows the dunes and the meadow",
            "no such pictures",
        );
        return;
    };
    let over_zone = source.same_place_as(&target, zone_middle);
    let is_under_drag = |stage: &Stage| stage.tree_of(1, |tree| tree.states(zone).contains(States::HOVER));
    let hit_over_zone = window_hit(stage, &target, zone_middle);

    source.button_down(on_dunes);
    source.drag_to(beside(on_dunes));
    source.drag_to(over_zone);
    stage.settle(ONE_CHANGE);
    let came_over = is_under_drag(stage);
    source.press(KeyStroke::ESCAPE);
    stage.settle(ONE_CHANGE);
    let let_go = !is_under_drag(stage);
    source.button_up(over_zone);
    stage.settle(ONE_CHANGE);
    stage.expectations.expect(
        came_over && let_go && stage.shows_text(1, "nothing dropped yet"),
        "Escape cancels the drag of a picture over the drop zone of the other window: the zone is let go and the release drops nothing",
        &format!(
            "the zone was under the drag: {came_over}, let go on Escape: {let_go}, holds {:?}; the pointer over the zone hits {hit_over_zone}",
            stage.tree_of(1, |tree| picture_in_zone(tree, zone))
        ),
    );

    source.button_down(on_meadow);
    source.drag_to(beside(on_meadow));
    source.drag_to(over_zone);
    stage.settle(ONE_CHANGE);
    let came_over = is_under_drag(stage);
    source.button_up(over_zone);
    stage.settle(ONE_CHANGE);
    let shown = stage.tree_of(1, |tree| picture_in_zone(tree, zone));
    stage.expectations.expect(
        came_over && shown == Some(gallery.image(GalleryPicture::Meadow)) && stage.shows_text(1, "dropped Meadow"),
        "the meadow dragged by the pointer from the first window drops on the zone of the second, which shows it by the image the first shows",
        &format!("the zone was under the drag: {came_over}, shows {shown:?}; the pointer over the zone hits {hit_over_zone}"),
    );
}

fn the_frame_closes_the_second_window_and_the_menu_bar_follows_the_first(stage: &mut Stage) {
    let (Some(first), Some(second)) = (stage.native_window(0), stage.native_window(1)) else {
        return;
    };
    second.performClose(None);
    stage.settle(ONE_CHANGE);
    let window_count = stage.catalog.windows().len();
    let is_on_screen = stage.native_window(1).is_some();
    stage.expectations.expect(
        window_count == 1 && !is_on_screen,
        "the button of the frame closes the second window, which leaves the catalog",
        &format!("{window_count} windows, the second on screen: {is_on_screen}"),
    );

    first.makeKeyWindow();
    stage.settle(ONE_CHANGE);
    let menus = menus_of_bar(stage.main_thread);
    stage.expectations.expect(
        menus == ["File"],
        "the menu bar shows the menus of the first window once it has the keyboard",
        &format!("{menus:?}"),
    );
    let Some(menu) = file_menu(stage.main_thread) else {
        return;
    };
    menu.performActionForItemAtIndex(0);
    stage.settle(SETTLING);
    let third = window_titled(stage.main_thread, THIRD_WINDOW_TITLE);
    stage.expectations.expect(
        stage.catalog.windows().len() == 2 && third.is_some(),
        "New Window of the menu bar runs the command of the first window",
        &format!("{} windows", stage.catalog.windows().len()),
    );
    if let Some(third) = third {
        third.performClose(None);
        stage.settle(ONE_CHANGE);
    }
    stage.expectations.expect(
        stage.catalog.windows().len() == 1,
        "the third window leaves the catalog when its frame closes it",
        &format!("{} windows", stage.catalog.windows().len()),
    );
}

fn the_scaffolding_page_stacks_a_split_view_vertically(stage: &mut Stage) {
    stage.click_text(0, Page::Scaffolding.title());
    let Some(split) = stage.tree_of(0, |tree| find_classed(tree, &[CLASSES.split, CLASSES.vertical])) else {
        stage.expectations.expect(
            false,
            "the page of the scaffolding shows a split view one pane above the other",
            "no such split view",
        );
        return;
    };
    stage.scroll_into_view(0, split);
    let (panes, dividers) = stage.tree_of(0, |tree| {
        let boxes_of = |class| {
            tree.children(split)
                .iter()
                .filter(|child| tree.classes(**child).contains(&class))
                .map(|child| tree.surface_box(*child))
                .collect::<Vec<Rect>>()
        };
        (boxes_of(CLASSES.split_pane), boxes_of(CLASSES.split_divider))
    });
    let is_stacked = match (panes.as_slice(), dividers.as_slice()) {
        ([upper, lower], [divider]) => {
            (upper.x - lower.x).abs() < LAYOUT_TOLERANCE
                && (upper.width - lower.width).abs() < LAYOUT_TOLERANCE
                && upper.bottom() <= divider.y + LAYOUT_TOLERANCE
                && divider.bottom() <= lower.y + LAYOUT_TOLERANCE
                && divider.width > divider.height
        }
        _ => false,
    };
    stage.expectations.expect(
        is_stacked && stage.is_in_view(0, split),
        "the vertical split view shows its panes one above the other, a horizontal line between them",
        &format!("panes {panes:?}, dividers {dividers:?}"),
    );
    let orientations: Vec<NSAccessibilityOrientation> = stage
        .accessible_elements(0)
        .iter()
        .filter(|element| element.role().as_deref() == Some("AXSplitter"))
        .map(AccessibleElement::orientation)
        .collect();
    stage.expectations.expect(
        orientations.contains(&NSAccessibilityOrientation::Horizontal),
        "the system sees the divider between the panes one above the other as a horizontal splitter",
        &format!("{orientations:?}"),
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
    let came_to_front = bring_to_front(main_thread);
    stage.expectations.expect(
        came_to_front,
        "the catalog comes to the front, where its windows take the keyboard and the menu bar",
        "the system kept another application in front",
    );
    stage.settle(SETTLING);

    the_catalog_launches_on_its_first_page(&mut stage);
    the_light_appearance_draws_the_light_values_of_the_tokens(&mut stage);
    a_click_presses_a_button(&mut stage);
    typing_fills_a_text_field(&mut stage);
    the_wheel_and_the_keys_scroll_a_million_rows(&mut stage);
    the_wheel_deep_in_the_rows_moves_them_by_one_pixel(&mut stage);
    the_chevron_shows_every_option_and_the_field_names_the_highlighted_one(&mut stage);
    the_images_page_shows_a_decoded_picture(&mut stage);
    a_popover_and_a_menu_open_over_the_window(&mut stage);
    the_theme_and_the_tokens_switch(&mut stage);
    a_second_window_opens_from_the_menu_bar(&mut stage);
    choosing_a_volume_and_a_view_redraws_the_disk_map(&mut stage);
    a_picture_goes_by_the_pointer_to_the_drop_zone_of_the_other_window(&mut stage);
    the_frame_closes_the_second_window_and_the_menu_bar_follows_the_first(&mut stage);
    the_scaffolding_page_stacks_a_split_view_vertically(&mut stage);

    stage.expectations.report("wire main path")
}
