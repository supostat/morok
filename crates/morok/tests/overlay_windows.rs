mod support;

use std::process::{self, ExitCode};
use std::thread;
use std::time::Duration;

use morok::components::{Button, ContextMenu, MenuItem, Popover, Select};
use morok::core::{ElementId, Point, Rect, Size};
use morok::reactive::Signal;
use morok::{Application, Styles, Window, WindowOptions};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSWindow};
use objc2_foundation::NSPoint;
use support::{EventSource, Expectations};

const CONTENT_SIZE: Size = Size {
    width: 400.0,
    height: 300.0,
};
/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// Long enough for the frames an input asks for to be drawn and for a window to open or close.
const ONE_CHANGE: Duration = Duration::from_millis(700);
/// A menu of the system that takes none of the keys of the test never closes by itself.
const MENU_PATIENCE: Duration = Duration::from_secs(30);
/// The virtual key codes of Escape, of the down arrow and of Return, with what each types.
const ESCAPE_KEY: (u16, &str) = (0x35, "\u{1b}");
const DOWN_KEY: (u16, &str) = (0x7D, "\u{f701}");
const RETURN_KEY: (u16, &str) = (0x24, "\r");
/// The room an overlay surface leaves around its panel, and the border and the padding of
/// a popover: the button inside the popover starts that far into the overlay surface.
const SHADOW_ROOM: f32 = 32.0;
const POPOVER_INSET: f32 = 17.0;
/// The distance between a popover and its anchor.
const POPOVER_GAP: f64 = 10.0;
/// The border and the padding of a dropdown list, and the height of one of its rows: the first
/// row starts that far into the panel of the list.
const LIST_INSET: f32 = 5.0;
const LIST_ROW_HEIGHT: f32 = 26.0;
/// How far a window may lie from where it is expected: windows lie on whole pixels.
const TOLERANCE: f64 = 0.5;
/// How far the opener starts from the right edge of the room of the screen when the window
/// stands at that edge: a panel 260 px wide that starts where the opener does is pushed
/// 150 px to the left.
const OPENER_TO_EDGE: f64 = 110.0;
/// How far the right edge pushes the panel at least.
const LEAST_PUSH: f64 = 100.0;
/// How far above the top edge of a popover panel a row runs through the half of its arrow
/// that sticks out of it, in points.
const ARROW_ROW: f64 = 3.0;
/// Where the middle of the arrow lies from the left edge of a panel nothing pushes: past the
/// border of 1 px, 28 px in, and half of the 10 px of the arrow.
const ARROW_MIDDLE: f64 = 34.0;
/// How far the middle of the arrow may lie from where it is expected: it is found in pixels.
const ARROW_TOLERANCE: f64 = 1.0;
/// The alpha from which a pixel of an overlay window is opaque: the arrow is, the shadow
/// around the panel is not.
const OPAQUE: u8 = 250;

/// What the root component of the window built.
#[derive(Clone, Copy, Debug)]
struct Built {
    opener: ElementId,
    elsewhere: ElementId,
    select: ElementId,
    /// The box a secondary press opens the menu of the system in.
    menu_area: ElementId,
}

/// The window of the test with what the expectations read of it.
struct Stage {
    main_thread: MainThreadMarker,
    application: Application,
    window: Window,
    source: EventSource,
    expectations: Expectations,
    built: Built,
    is_open: Signal<bool>,
    cleanups: Signal<u32>,
    chosen: Signal<Vec<&'static str>>,
    /// The option the select holds: its place among the three.
    sorting: Signal<usize>,
}

impl Stage {
    /// A window on a button that opens a popover with a button of its own, another button,
    /// a select of three options that holds the second, and a box with a context menu.
    fn open(main_thread: MainThreadMarker) -> Self {
        let application = Application::new(Styles::of_kit()).expect("this Mac runs Metal 4");
        let (is_open, cleanups, chosen, sorting) = application.update(|runtime| {
            (
                runtime.signal(false),
                runtime.signal(0_u32),
                runtime.signal(Vec::new()),
                runtime.signal(1_usize),
            )
        });
        let options = WindowOptions {
            title: String::from("morok: overlay windows"),
            content_size: CONTENT_SIZE,
            blur_behind: false,
        };
        let mut built = None;
        let window = application.open_window(&options, |mount| {
            let opener = Button::new("Open")
                .on_press(move |runtime| runtime.update(is_open, |is_open| *is_open = !*is_open))
                .mount(mount);
            Popover::new(is_open)
                .label("Clean up")
                .content(move |mount| {
                    Button::new("Clean up")
                        .on_press(move |runtime| runtime.update(cleanups, |cleanups| *cleanups += 1))
                        .mount(mount)
                })
                .attach(mount, opener);
            let elsewhere = Button::new("Elsewhere").mount(mount);
            let select = Select::new(sorting)
                .option(0, "By name")
                .option(1, "By size")
                .option(2, "By kind")
                .label("Sort order")
                .mount(mount);
            let files = Button::new("Files").mount(mount);
            let command = |label: &'static str, name: &'static str| {
                MenuItem::new(label).on_choose(move |runtime| runtime.update(chosen, |chosen| chosen.push(name)))
            };
            let menu_area =
                ContextMenu::new(vec![command("Reveal", "reveal"), command("Rename", "rename")]).mount(mount, files);
            built = Some(Built {
                opener,
                elsewhere,
                select,
                menu_area,
            });
            mount
                .element()
                .child(opener)
                .child(elsewhere)
                .child(select)
                .child(menu_area)
                .id()
        });
        Self {
            main_thread,
            application,
            window,
            source: EventSource::for_the_window(main_thread, CONTENT_SIZE),
            expectations: Expectations::default(),
            built: built.expect("the window mounted its root component"),
            is_open,
            cleanups,
            chosen,
            sorting,
        }
    }

    fn box_of(&self, element: ElementId) -> Rect {
        self.window.with_tree(|tree| tree.border_box(element))
    }

    fn middle_of(&self, element: ElementId) -> Point {
        let shown_in = self.box_of(element);
        Point {
            x: shown_in.x + shown_in.width / 2.0,
            y: shown_in.y + shown_in.height / 2.0,
        }
    }

    fn click(&self, element: ElementId) {
        self.source.click(self.middle_of(element));
        self.application.run_for(ONE_CHANGE);
    }

    /// The window of the overlay opened over the window of the test, when one is on screen.
    fn overlay_window(&self) -> Option<Retained<NSWindow>> {
        NSApplication::sharedApplication(self.main_thread)
            .windows()
            .to_vec()
            .into_iter()
            .find(|window| window.parentWindow().is_some())
    }

    /// The source of the events of the overlay window.
    fn overlay_source(&self) -> Option<EventSource> {
        let overlay = self.overlay_window()?;
        let frame = overlay.frame().size;
        let surface = Size {
            width: frame.width as f32,
            height: frame.height as f32,
        };
        Some(EventSource::for_window(overlay, surface))
    }

    fn is_open(&self) -> bool {
        self.application.update(|runtime| runtime.get(self.is_open))
    }

    fn sorting(&self) -> usize {
        self.application.update(|runtime| runtime.get(self.sorting))
    }
}

fn a_press_of_the_opener_shows_the_popover_in_a_window_under_it(stage: &mut Stage) {
    stage.click(stage.built.opener);

    let overlay = stage.overlay_window();
    stage.expectations.expect(
        stage.is_open() && overlay.is_some(),
        "an open popover has a window of its own over the window of its anchor",
        &format!("open: {}, on screen: {}", stage.is_open(), overlay.is_some()),
    );
    let Some(overlay) = overlay else {
        return;
    };
    let window = stage.source.window();
    let surface = window.contentRectForFrameRect(window.frame());
    let anchor = stage.box_of(stage.built.opener);
    // The desktop is measured upwards and the surface downwards from its top.
    let anchor_bottom = surface.origin.y + surface.size.height - f64::from(anchor.bottom());
    let frame = overlay.frame();
    let panel_left = frame.origin.x + f64::from(SHADOW_ROOM);
    let panel_top = frame.origin.y + frame.size.height - f64::from(SHADOW_ROOM);
    let expected_left = surface.origin.x + f64::from(anchor.x);
    let expected_top = anchor_bottom - POPOVER_GAP;
    stage.expectations.expect(
        (panel_left - expected_left).abs() <= TOLERANCE && (panel_top - expected_top).abs() <= TOLERANCE,
        "the panel of the popover lies under its anchor, starting where the anchor starts",
        &format!("the panel starts at ({panel_left}, {panel_top}), expected at ({expected_left}, {expected_top})"),
    );
    stage.expectations.expect(
        (frame.size.width - f64::from(260.0 + 2.0 * SHADOW_ROOM)).abs() <= TOLERANCE,
        "the window of the popover is as wide as its panel and the room around it",
        &format!("{:?}", frame.size),
    );
}

fn a_press_inside_the_popover_reaches_its_content_and_keeps_it_open(stage: &mut Stage) {
    let Some(overlay) = stage.overlay_source() else {
        return;
    };
    let inside_the_button = Point {
        x: SHADOW_ROOM + POPOVER_INSET + 20.0,
        y: SHADOW_ROOM + POPOVER_INSET + 14.0,
    };

    overlay.click(inside_the_button);
    stage.application.run_for(ONE_CHANGE);

    let cleanups = stage.application.update(|runtime| runtime.get(stage.cleanups));
    stage.expectations.expect(
        cleanups == 1 && stage.is_open(),
        "a click on a button of a popover presses the button and leaves the popover open",
        &format!("the button was pressed {cleanups} times, open: {}", stage.is_open()),
    );
}

fn escape_in_the_popover_closes_its_window(stage: &mut Stage) {
    let Some(overlay) = stage.overlay_source() else {
        return;
    };

    overlay.press_key(ESCAPE_KEY.0, ESCAPE_KEY.1);
    stage.application.run_for(ONE_CHANGE);

    stage.expectations.expect(
        !stage.is_open() && stage.overlay_window().is_none(),
        "Escape in a popover closes it and takes its window off the screen",
        &format!(
            "open: {}, on screen: {}",
            stage.is_open(),
            stage.overlay_window().is_some()
        ),
    );
}

fn a_press_outside_the_popover_closes_its_window(stage: &mut Stage) {
    stage.click(stage.built.opener);
    stage.expectations.expect(
        stage.overlay_window().is_some(),
        "a popover opens again after it was closed",
        "no overlay window is on screen",
    );

    stage.click(stage.built.elsewhere);

    stage.expectations.expect(
        !stage.is_open() && stage.overlay_window().is_none(),
        "a press on the window under a popover closes the popover",
        &format!(
            "open: {}, on screen: {}",
            stage.is_open(),
            stage.overlay_window().is_some()
        ),
    );
}

fn a_popover_the_right_edge_of_the_screen_pushes_points_its_arrow_at_its_opener(stage: &mut Stage) {
    let window = stage.source.window();
    let Some(room) = window.screen().map(|screen| screen.visibleFrame()) else {
        return;
    };
    let first_place = window.frame().origin;
    let opener = stage.box_of(stage.built.opener);
    let room_right = room.origin.x + room.size.width;
    window.setFrameOrigin(NSPoint::new(
        room_right - OPENER_TO_EDGE - f64::from(opener.x),
        first_place.y,
    ));
    stage.application.run_for(ONE_CHANGE);
    let opener_left = window.contentRectForFrameRect(window.frame()).origin.x + f64::from(opener.x);

    stage.click(stage.built.opener);

    let overlay = stage.overlay_window();
    let panel_left = overlay
        .as_ref()
        .map(|overlay| overlay.frame().origin.x + f64::from(SHADOW_ROOM));
    stage.expectations.expect(
        panel_left.is_some_and(|panel_left| panel_left < opener_left - LEAST_PUSH),
        "the right edge of the screen pushes the panel of the popover to the left of its opener",
        &format!("the panel starts at {panel_left:?}, the opener at {opener_left}"),
    );
    let arrow_middle = overlay.as_ref().and_then(|overlay| {
        let frame = overlay.frame();
        let panel_top = frame.origin.y + frame.size.height - f64::from(SHADOW_ROOM);
        let row = support::screen_row(
            overlay,
            NSPoint::new(frame.origin.x, panel_top + ARROW_ROW),
            frame.size.width,
            false,
            stage.main_thread,
        )?;
        Some(frame.origin.x + middle_of_opaque(&row)?)
    });
    let pointed_at = opener_left + ARROW_MIDDLE;
    stage.expectations.expect(
        arrow_middle.is_some_and(|middle| (middle - pointed_at).abs() <= ARROW_TOLERANCE),
        "the arrow of the pushed popover points at its opener where it would if nothing pushed it",
        &format!("the arrow is at {arrow_middle:?}, expected at {pointed_at}"),
    );

    stage.click(stage.built.opener);
    window.setFrameOrigin(first_place);
    stage.application.run_for(ONE_CHANGE);
}

/// The middle of the opaque pixels of a row, in pixels from its start; none when no pixel of
/// it is opaque.
fn middle_of_opaque(row: &[[u8; 4]]) -> Option<f64> {
    let first = row.iter().position(|pixel| pixel[3] >= OPAQUE)?;
    let last = row.iter().rposition(|pixel| pixel[3] >= OPAQUE)?;
    Some((first + last + 1) as f64 / 2.0)
}

fn the_list_of_a_select_opens_in_a_window_that_takes_the_pointer_and_never_the_keyboard(stage: &mut Stage) {
    stage.click(stage.built.select);

    let overlay = stage.overlay_window();
    stage.expectations.expect(
        overlay.is_some(),
        "an open select has a window of its own for its list",
        "no overlay window is on screen",
    );
    let Some(overlay) = overlay else {
        return;
    };
    stage.expectations.expect(
        !overlay.canBecomeKeyWindow() && !overlay.isKeyWindow() && !overlay.ignoresMouseEvents(),
        "the window of the list takes the pointer and leaves the keyboard with the window of the select",
        &format!(
            "can take the keyboard: {}, has it: {}, lets the pointer through: {}",
            overlay.canBecomeKeyWindow(),
            overlay.isKeyWindow(),
            overlay.ignoresMouseEvents()
        ),
    );
    let width = overlay.frame().size.width;
    let expected_width = f64::from(stage.box_of(stage.built.select).width + 2.0 * SHADOW_ROOM);
    stage.expectations.expect(
        (width - expected_width).abs() <= TOLERANCE,
        "the list is as wide as the select, with the room around it",
        &format!("the window of the list is {width} wide, expected {expected_width}"),
    );
}

fn the_keys_of_the_window_of_a_select_choose_from_its_open_list(stage: &mut Stage) {
    for (key_code, characters) in [DOWN_KEY, RETURN_KEY] {
        stage.source.press_key(key_code, characters);
        stage.application.run_for(ONE_CHANGE);
    }

    stage.expectations.expect(
        stage.sorting() == 2 && stage.overlay_window().is_none(),
        "the down arrow and Return in the window of the select choose the option after the held one and close the list",
        &format!(
            "the select holds option {}, on screen: {}",
            stage.sorting(),
            stage.overlay_window().is_some()
        ),
    );
}

fn a_click_on_a_row_of_the_list_of_a_select_chooses_its_option(stage: &mut Stage) {
    stage.click(stage.built.select);
    let Some(overlay) = stage.overlay_source() else {
        stage.expectations.expect(
            false,
            "a select opens its list again after it was closed",
            "no overlay window is on screen",
        );
        return;
    };
    let inside_the_first_row = Point {
        x: SHADOW_ROOM + LIST_INSET + 40.0,
        y: SHADOW_ROOM + LIST_INSET + LIST_ROW_HEIGHT / 2.0,
    };

    overlay.click(inside_the_first_row);
    stage.application.run_for(ONE_CHANGE);

    stage.expectations.expect(
        stage.sorting() == 0 && stage.overlay_window().is_none(),
        "a click on a row of the list chooses its option and closes the list",
        &format!(
            "the select holds option {}, on screen: {}",
            stage.sorting(),
            stage.overlay_window().is_some()
        ),
    );
}

/// The menu of the system takes the keys that wait in the queue of the application while it
/// is open: the down arrow highlights its first command and Return chooses it.
fn a_secondary_press_shows_the_menu_of_the_system_and_its_command_runs(stage: &mut Stage) {
    thread::spawn(|| {
        thread::sleep(MENU_PATIENCE);
        eprintln!("overlay windows failed:\nthe menu of the system took none of the keys queued for it");
        process::exit(1);
    });

    stage.source.secondary_click(stage.middle_of(stage.built.menu_area));
    for (key_code, characters) in [DOWN_KEY, RETURN_KEY] {
        stage.source.queue_key_press(stage.main_thread, key_code, characters);
    }
    stage.application.run_for(ONE_CHANGE);
    stage.application.run_for(ONE_CHANGE);

    let chosen = stage.application.update(|runtime| runtime.get(stage.chosen));
    stage.expectations.expect(
        chosen == ["reveal"],
        "the command chosen in the menu of the system runs",
        &format!("{chosen:?}"),
    );
    stage.expectations.expect(
        stage.overlay_window().is_none(),
        "the menu of the system needs no overlay of the kit",
        "an overlay window is on screen",
    );
}

fn main() -> ExitCode {
    let main_thread = MainThreadMarker::new().expect("a test that opens a window runs on the main thread");
    let mut stage = Stage::open(main_thread);
    stage.application.run_for(SETTLING);

    a_press_of_the_opener_shows_the_popover_in_a_window_under_it(&mut stage);
    a_press_inside_the_popover_reaches_its_content_and_keeps_it_open(&mut stage);
    escape_in_the_popover_closes_its_window(&mut stage);
    a_press_outside_the_popover_closes_its_window(&mut stage);
    a_popover_the_right_edge_of_the_screen_pushes_points_its_arrow_at_its_opener(&mut stage);
    the_list_of_a_select_opens_in_a_window_that_takes_the_pointer_and_never_the_keyboard(&mut stage);
    the_keys_of_the_window_of_a_select_choose_from_its_open_list(&mut stage);
    a_click_on_a_row_of_the_list_of_a_select_chooses_its_option(&mut stage);
    a_secondary_press_shows_the_menu_of_the_system_and_its_command_runs(&mut stage);

    stage.expectations.report("overlay windows")
}
