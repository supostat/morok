mod support;

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Duration;

use morok::components::Button;
use morok::core::{ElementId, Key, KeyChord, Modifiers, Point, Size};
use morok::reactive::Signal;
use morok::{Application, Styles, Window, WindowOptions};
use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSBackingStoreType, NSColor, NSEventModifierFlags, NSMenu, NSScreen, NSVisualEffectView, NSWindow,
    NSWindowStyleMask,
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_core_graphics::{
    CGBitmapContextCreate, CGColorSpace, CGContext, CGImage, CGImageAlphaInfo, CGWindowImageOption, CGWindowListOption,
    kCGColorSpaceSRGB,
};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use support::{EventSource, Expectations};

const CONTENT_SIZE: Size = Size {
    width: 400.0,
    height: 300.0,
};
const FILES_TITLE: &str = "morok: files";
const INSPECTOR_TITLE: &str = "morok: inspector";
const BLURRED_TITLE: &str = "morok: blurred";
const BLURRED_OVER_EDGE_TITLE: &str = "morok: blurred over the edge";
const PLAIN_OVER_EDGE_TITLE: &str = "morok: plain over the edge";
/// The size of a window that draws nothing over the edge of the backdrop.
const OVER_EDGE_SIZE: Size = Size {
    width: 200.0,
    height: 80.0,
};
/// The size of each half of the backdrop, in points.
const BACKDROP_HALF: NSSize = NSSize::new(240.0, 240.0);
/// Where the backdrop lies from the bottom left corner of the room the main screen has for
/// windows, in points.
const BACKDROP_INSET: f64 = 80.0;
/// How high above the bottom of the backdrop the window that asks for blur stands, in points.
const BLURRED_HEIGHT: f64 = 130.0;
/// How high above the bottom of the backdrop the window that asks for none stands, in points.
const PLAIN_HEIGHT: f64 = 10.0;
/// How far the captured row reaches on either side of the edge, in points.
const ROW_REACH: f64 = 24.0;
/// How far apart the red and blue of a pixel are at least when its colour leans to one of them.
const LEAN: i32 = 8;
/// How many pixels of a row wide at least the change from one colour to the other is when the
/// window blurs what is behind it.
const BLUR_WIDTH: usize = 3;
/// How far apart two pixels of the background of a window are at most in each channel.
const SAME_BACKGROUND: i32 = 6;
const RED: [f64; 3] = [1.0, 0.0, 0.0];
const BLUE: [f64; 3] = [0.0, 0.0, 1.0];
/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// Long enough for the frames an input asks for to be drawn and for a window to open or close.
const ONE_CHANGE: Duration = Duration::from_millis(700);
/// The virtual key code of the space bar, with what it types.
const SPACE_KEY: (u16, &str) = (0x31, " ");
/// The chord of the command that opens the inspector.
const NEW_WINDOW: KeyChord = KeyChord {
    key: Key::Character('n'),
    modifiers: Modifiers::COMMAND,
};
/// The chord of the command of the inspector.
const REFRESH: KeyChord = KeyChord {
    key: Key::Character('r'),
    modifiers: Modifiers::COMMAND,
};

/// The buttons of the window of the files.
#[derive(Clone, Copy, Debug)]
struct FilesButtons {
    reveal: ElementId,
    close_others: ElementId,
}

/// The windows of the test with what the expectations read of them.
struct Stage {
    main_thread: MainThreadMarker,
    application: Application,
    /// The window the application opens first.
    files: Window,
    files_buttons: FilesButtons,
    /// The windows a handler of the first window opened, in the order it opened them, each
    /// with the button it shows.
    opened: Rc<RefCell<Vec<(Window, ElementId)>>>,
    /// The buttons pressed, by the name of the window of each.
    presses: Signal<Vec<&'static str>>,
    expectations: Expectations,
}

fn options(title: &str, blur_behind: bool) -> WindowOptions {
    WindowOptions {
        title: String::from(title),
        content_size: CONTENT_SIZE,
        blur_behind,
    }
}

impl Stage {
    /// The window of the files: a button that notes its press, and a button whose handler
    /// closes the windows the application opened after this one. Its root registers the
    /// command that opens the inspector, in the menu "File" of the menu bar; the root of the
    /// inspector registers a command of its own, in the menu "Inspector".
    fn open(main_thread: MainThreadMarker) -> Self {
        let application = Application::new(Styles::of_kit()).expect("this Mac runs Metal 4");
        let presses = application.update(|runtime| runtime.signal(Vec::new()));
        let opened: Rc<RefCell<Vec<(Window, ElementId)>>> = Rc::default();

        let open_inspector = {
            let (application, opened) = (application.clone(), Rc::clone(&opened));
            move || {
                let mut inspect = None;
                let window = application.open_window(&options(INSPECTOR_TITLE, false), |mount| {
                    mount
                        .surface_root()
                        .menu_command("Inspector", "Refresh", REFRESH, |_| {});
                    let line = mount.text("Inspector of quartz").id();
                    let button = Button::new("Reveal")
                        .on_press(move |runtime| runtime.update(presses, |presses| presses.push("inspector")))
                        .mount(mount);
                    inspect = Some(button);
                    mount.element().child(line).child(button).id()
                });
                let inspect = inspect.expect("the window mounted its root component");
                opened.borrow_mut().push((window, inspect));
            }
        };
        let close_opened = {
            let opened = Rc::clone(&opened);
            move || opened.borrow_mut().clear()
        };

        let mut buttons = None;
        let files = application.open_window(&options(FILES_TITLE, false), |mount| {
            let deferring = application.clone();
            mount
                .surface_root()
                .menu_command("File", "New Window", NEW_WINDOW, move |_| {
                    deferring.defer(open_inspector.clone());
                });
            let line = mount.text("Files").id();
            let reveal = Button::new("Reveal")
                .on_press(move |runtime| runtime.update(presses, |presses| presses.push("files")))
                .mount(mount);
            let closing = application.clone();
            let close_others = Button::new("Close")
                .on_press(move |_| closing.defer(close_opened.clone()))
                .mount(mount);
            buttons = Some(FilesButtons { reveal, close_others });
            mount.element().child(line).child(reveal).child(close_others).id()
        });
        Self {
            main_thread,
            application,
            files,
            files_buttons: buttons.expect("the window mounted its root component"),
            opened,
            presses,
            expectations: Expectations::default(),
        }
    }

    /// The window of the system that carries the title, while it is on screen.
    fn native_window(&self, title: &str) -> Option<Retained<NSWindow>> {
        NSApplication::sharedApplication(self.main_thread)
            .windows()
            .to_vec()
            .into_iter()
            .find(|window| window.isVisible() && window.title().to_string() == title)
    }

    /// The source of the events of the window that carries the title.
    fn events_of(&self, title: &str) -> Option<EventSource> {
        Some(EventSource::for_window(self.native_window(title)?, CONTENT_SIZE))
    }

    fn presses(&self) -> Vec<&'static str> {
        self.application.update(|runtime| runtime.get(self.presses))
    }

    /// The menu "File" of the menu bar.
    fn file_menu(&self) -> Option<Retained<NSMenu>> {
        let bar = NSApplication::sharedApplication(self.main_thread).mainMenu()?;
        (0..bar.numberOfItems())
            .filter_map(|position| bar.itemAtIndex(position))
            .find(|item| item.title().to_string() == "File")
            .and_then(|item| item.submenu())
    }

    /// The titles of the menus of the menu bar after the menu of the application.
    fn menus_of_bar(&self) -> Vec<String> {
        let Some(bar) = NSApplication::sharedApplication(self.main_thread).mainMenu() else {
            return Vec::new();
        };
        (1..bar.numberOfItems())
            .filter_map(|position| bar.itemAtIndex(position))
            .map(|item| item.title().to_string())
            .collect()
    }

    /// Opens a window that draws nothing, of the size of a window over the edge of the backdrop.
    fn open_empty(&self, title: &str, blur_behind: bool) -> Window {
        let options = WindowOptions {
            title: String::from(title),
            content_size: OVER_EDGE_SIZE,
            blur_behind,
        };
        self.application.open_window(&options, |mount| mount.element().id())
    }
}

/// Two windows of one vivid colour each, with no frame and no shadow, side by side: red on the
/// left, blue on the right, meeting at a sharp vertical edge. They close with the value.
struct Backdrop {
    halves: [Retained<NSWindow>; 2],
}

impl Backdrop {
    /// The halves with the bottom left corner of the red one at `origin` on the screen.
    fn open(main_thread: MainThreadMarker, origin: NSPoint) -> Self {
        let half = |[red, green, blue]: [f64; 3], left: f64| {
            let frame = NSRect::new(NSPoint::new(left, origin.y), BACKDROP_HALF);
            // SAFETY: the window is created on the main thread, and it is not released when
            // closed: the backdrop owns it.
            let window = unsafe {
                let window = NSWindow::initWithContentRect_styleMask_backing_defer(
                    NSWindow::alloc(main_thread),
                    frame,
                    NSWindowStyleMask::Borderless,
                    NSBackingStoreType::Buffered,
                    false,
                );
                window.setReleasedWhenClosed(false);
                window
            };
            window.setBackgroundColor(Some(&NSColor::colorWithSRGBRed_green_blue_alpha(red, green, blue, 1.0)));
            window.setHasShadow(false);
            window.orderFront(None);
            window
        };
        Self {
            halves: [half(RED, origin.x), half(BLUE, origin.x + BACKDROP_HALF.width)],
        }
    }

    /// Where the halves meet, in points from the left of the main screen.
    fn edge(&self) -> f64 {
        self.halves[1].frame().origin.x
    }

    fn bottom(&self) -> f64 {
        self.halves[0].frame().origin.y
    }
}

impl Drop for Backdrop {
    fn drop(&mut self) {
        for half in &self.halves {
            half.close();
        }
    }
}

/// The pixels the screen shows of `window` and of the windows under it along one row across
/// `edge`, in the middle of the content of the window: one pixel to a point, from `ROW_REACH`
/// points left of the edge to as far right of it, red, green, blue and alpha each. None when
/// the system hands no picture of the screen.
fn row_across(window: &NSWindow, edge: f64, main_thread: MainThreadMarker) -> Option<Vec<[u8; 4]>> {
    let screen_height = NSScreen::screens(main_thread).firstObject()?.frame().size.height;
    let content = window.contentRectForFrameRect(window.frame());
    let middle = content.origin.y + content.size.height / 2.0;
    let row = CGRect::new(
        CGPoint::new(edge - ROW_REACH, screen_height - middle),
        CGSize::new(2.0 * ROW_REACH, 1.0),
    );
    #[expect(deprecated, reason = "the workspace has no binding of ScreenCaptureKit")]
    let shown = objc2_core_graphics::CGWindowListCreateImage(
        row,
        CGWindowListOption::OptionOnScreenBelowWindow | CGWindowListOption::OptionIncludingWindow,
        u32::try_from(window.windowNumber()).ok()?,
        CGWindowImageOption::NominalResolution,
    )?;
    let pixels = pixels_of(&shown)?;
    Some(pixels.into_iter().take(CGImage::width(Some(&shown))).collect())
}

/// The pixels of an image drawn whole into a bitmap of its size, rows from the top.
fn pixels_of(image: &CGImage) -> Option<Vec<[u8; 4]>> {
    let (width, height) = (CGImage::width(Some(image)), CGImage::height(Some(image)));
    let mut pixels = vec![[0_u8; 4]; width * height];
    // SAFETY: the name is a constant of Core Graphics.
    let color_space = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB }));
    // SAFETY: the buffer holds `height` rows of `width` pixels of red, green, blue and alpha,
    // premultiplied, and outlives the context, which is dropped before it is read.
    let context = unsafe {
        CGBitmapContextCreate(
            pixels.as_mut_ptr().cast::<c_void>(),
            width,
            height,
            8,
            width * 4,
            color_space.as_deref(),
            CGImageAlphaInfo::PremultipliedLast.0,
        )
    }?;
    let whole = CGRect::new(CGPoint::ZERO, CGSize::new(width as f64, height as f64));
    CGContext::draw_image(Some(&context), whole, Some(image));
    drop(context);
    Some(pixels)
}

/// How much redder than blue the pixel is; negative when it is bluer.
fn red_over_blue(pixel: [u8; 4]) -> i32 {
    i32::from(pixel[0]) - i32::from(pixel[2])
}

/// How many pixels of the row lie between its ends in how red they are over blue: neither
/// near the one end nor near the other. None when the ends do not lean to the colours of the
/// halves under them, red on the left and blue on the right.
fn blur_width(row: &[[u8; 4]]) -> Option<usize> {
    let (left, right) = (red_over_blue(*row.first()?), red_over_blue(*row.last()?));
    if left < LEAN || right > -LEAN {
        return None;
    }
    let margin = (left - right) / 10;
    Some(
        row.iter()
            .map(|pixel| red_over_blue(*pixel))
            .filter(|lean| *lean < left - margin && *lean > right + margin)
            .count(),
    )
}

/// Whether the ends of the row show one colour.
fn shows_one_colour(row: &[[u8; 4]]) -> bool {
    let (Some(first), Some(last)) = (row.first(), row.last()) else {
        return false;
    };
    first
        .iter()
        .zip(last)
        .all(|(one, other)| (i32::from(*one) - i32::from(*other)).abs() <= SAME_BACKGROUND)
}

fn middle_of(window: &Window, element: ElementId) -> Point {
    let shown_in = window.with_tree(|tree| tree.border_box(element));
    Point {
        x: shown_in.x + shown_in.width / 2.0,
        y: shown_in.y + shown_in.height / 2.0,
    }
}

fn a_registered_shortcut_stands_in_the_menu_bar_with_its_chord(stage: &mut Stage) {
    let menus = stage.menus_of_bar();
    stage.expectations.expect(
        menus == ["File"],
        "the menu bar holds the menu of the application and the menu the shortcut names",
        &format!("{menus:?}"),
    );
    let line = stage.file_menu().and_then(|menu| menu.itemAtIndex(0));
    let shown = line.as_ref().map(|line| {
        (
            line.title().to_string(),
            line.keyEquivalent().to_string(),
            line.keyEquivalentModifierMask(),
        )
    });
    stage.expectations.expect(
        shown
            == Some((
                String::from("New Window"),
                String::from("n"),
                NSEventModifierFlags::Command,
            )),
        "the line of the shortcut carries its title and its chord",
        &format!("{shown:?}"),
    );
}

fn the_line_of_the_menu_bar_runs_the_handler_which_opens_a_further_window(stage: &mut Stage) {
    let glyphs_before = stage.application.atlas_statistics().mask;
    let Some(menu) = stage.file_menu() else {
        return;
    };

    menu.performActionForItemAtIndex(0);
    stage.application.run_for(SETTLING);

    let opened = stage.opened.borrow().len();
    stage.expectations.expect(
        opened == 1 && stage.native_window(INSPECTOR_TITLE).is_some(),
        "choosing the line opens the window its handler asks the application for",
        &format!(
            "the handler opened {opened} windows, on screen: {}",
            stage.native_window(INSPECTOR_TITLE).is_some()
        ),
    );
    let Some((inspector, _)) = stage
        .opened
        .borrow()
        .first()
        .map(|(window, button)| (window.frame_statistics(), *button))
    else {
        return;
    };
    let glyphs_after = stage.application.atlas_statistics().mask;
    stage.expectations.expect(
        inspector.presented_frames > 0
            && glyphs_after.rasterized_glyphs > glyphs_before.rasterized_glyphs
            && glyphs_after.pages == glyphs_before.pages,
        "the further window draws its new glyphs into the atlas the windows of the application share",
        &format!("the window presented {inspector:?}; the atlas went from {glyphs_before:?} to {glyphs_after:?}"),
    );
}

fn two_windows_keep_trees_and_focus_of_their_own(stage: &mut Stage) {
    let Some(inspect) = stage.opened.borrow().first().map(|(_, button)| *button) else {
        return;
    };
    let (Some(files_events), Some(inspector_events)) = (stage.events_of(FILES_TITLE), stage.events_of(INSPECTOR_TITLE))
    else {
        return;
    };
    let reveal = stage.files_buttons.reveal;
    let texts = |window: &Window| {
        window.with_tree(|tree| {
            let root = tree.children(tree.root())[0];
            tree.text(tree.children(root)[0]).to_owned()
        })
    };
    let inspector_text = texts(&stage.opened.borrow()[0].0);
    stage.expectations.expect(
        texts(&stage.files) == "Files" && inspector_text == "Inspector of quartz",
        "each window shows the tree its root component built",
        &format!(
            "the first shows `{}`, the second `{inspector_text}`",
            texts(&stage.files)
        ),
    );

    files_events.click(middle_of(&stage.files, reveal));
    stage.application.run_for(ONE_CHANGE);
    inspector_events.click(middle_of(&stage.opened.borrow()[0].0, inspect));
    stage.application.run_for(ONE_CHANGE);

    let focus_of_files = stage.files.with_tree(|tree| tree.focused());
    let focus_of_inspector = stage.opened.borrow()[0].0.with_tree(|tree| tree.focused());
    stage.expectations.expect(
        focus_of_files == Some(reveal) && focus_of_inspector == Some(inspect),
        "a press in one window leaves the focus of the other where it was",
        &format!("the first has focus on {focus_of_files:?}, the second on {focus_of_inspector:?}"),
    );
    stage.expectations.expect(
        stage.presses() == ["files", "inspector"],
        "each window presses its own button",
        &format!("{:?}", stage.presses()),
    );

    inspector_events.press_key(SPACE_KEY.0, SPACE_KEY.1);
    stage.application.run_for(ONE_CHANGE);
    files_events.press_key(SPACE_KEY.0, SPACE_KEY.1);
    stage.application.run_for(ONE_CHANGE);

    stage.expectations.expect(
        stage.presses() == ["files", "inspector", "inspector", "files"],
        "a key goes to the focused button of the window it is typed in",
        &format!("{:?}", stage.presses()),
    );
}

fn the_menu_bar_shows_the_menus_of_the_window_that_has_the_keyboard(stage: &mut Stage) {
    let (Some(files), Some(inspector)) = (stage.native_window(FILES_TITLE), stage.native_window(INSPECTOR_TITLE))
    else {
        return;
    };
    let mut shown = Vec::new();
    for (window, menu) in [
        (&inspector, "Inspector"),
        (&files, "File"),
        (&inspector, "Inspector"),
        (&files, "File"),
    ] {
        window.makeKeyWindow();
        stage.application.run_for(ONE_CHANGE);
        shown.push((menu, stage.menus_of_bar()));
    }
    stage.expectations.expect(
        shown.iter().all(|(menu, menus)| menus == &[*menu]),
        "after the next frame the menu bar shows the menus of the window that took the keyboard, each time",
        &format!("the menu each window names, with the menus the bar showed then: {shown:?}"),
    );
}

fn a_handler_closes_the_further_window_and_the_first_goes_on(stage: &mut Stage) {
    let Some(files_events) = stage.events_of(FILES_TITLE) else {
        return;
    };
    let presses_before = stage.presses().len();

    files_events.click(middle_of(&stage.files, stage.files_buttons.close_others));
    stage.application.run_for(ONE_CHANGE);

    stage.expectations.expect(
        stage.opened.borrow().is_empty() && stage.native_window(INSPECTOR_TITLE).is_none(),
        "a window the application drops leaves the screen",
        &format!(
            "the application holds {} further windows, on screen: {}",
            stage.opened.borrow().len(),
            stage.native_window(INSPECTOR_TITLE).is_some()
        ),
    );

    files_events.click(middle_of(&stage.files, stage.files_buttons.reveal));
    stage.application.run_for(ONE_CHANGE);
    stage.expectations.expect(
        stage.presses().len() == presses_before + 1 && stage.file_menu().is_some(),
        "the first window answers on, and the menu bar keeps its menus",
        &format!("{:?}", stage.presses()),
    );
}

fn the_frame_closes_a_window_and_the_application_hears_of_it(stage: &mut Stage) {
    let Some(menu) = stage.file_menu() else {
        return;
    };
    menu.performActionForItemAtIndex(0);
    stage.application.run_for(SETTLING);
    let heard = Rc::new(Cell::new(0));
    {
        let opened = stage.opened.borrow();
        let Some((inspector, _)) = opened.first() else {
            return;
        };
        let hearing = Rc::clone(&heard);
        inspector.on_closed(move || hearing.set(hearing.get() + 1));
    }
    let (Some(inspector), Some(files), Some(files_events)) = (
        stage.native_window(INSPECTOR_TITLE),
        stage.native_window(FILES_TITLE),
        stage.events_of(FILES_TITLE),
    ) else {
        return;
    };

    inspector.performClose(None);
    let heard_at_once = heard.get();
    stage.application.run_for(ONE_CHANGE);

    let is_open = stage.opened.borrow()[0].0.is_open();
    let is_on_screen = stage.native_window(INSPECTOR_TITLE).is_some();
    stage.expectations.expect(
        heard_at_once == 0 && heard.get() == 1 && !is_open && !is_on_screen,
        "the button of the frame closes the window, and its handler hears of it on the next turn of the run loop",
        &format!(
            "heard {heard_at_once} times at once and {} times after; open: {is_open}, on screen: {is_on_screen}",
            heard.get()
        ),
    );

    files.makeKeyWindow();
    stage.application.run_for(ONE_CHANGE);
    let menus_once_key = stage.menus_of_bar();
    stage.expectations.expect(
        menus_once_key == ["File"],
        "the menu bar shows the menus of the first window once that one has the keyboard",
        &format!("{menus_once_key:?}"),
    );

    let presses_before = stage.presses().len();
    stage.opened.borrow_mut().clear();
    stage.application.run_for(ONE_CHANGE);
    files_events.click(middle_of(&stage.files, stage.files_buttons.reveal));
    stage.application.run_for(ONE_CHANGE);
    let menus_after_drop = stage.menus_of_bar();
    stage.expectations.expect(
        heard.get() == 1
            && menus_after_drop == ["File"]
            && stage.presses().len() == presses_before + 1
            && stage.native_window(FILES_TITLE).is_some(),
        "dropping the closed window afterwards changes nothing: the first window answers on and keeps the menu bar",
        &format!(
            "heard {} times; the bar shows {menus_after_drop:?}; presses {:?}",
            heard.get(),
            stage.presses()
        ),
    );
}

fn a_window_that_asks_for_it_lies_over_the_blur_of_the_system(stage: &mut Stage) {
    let pressed = stage.presses;
    let mut button = None;
    let blurred = stage.application.open_window(&options(BLURRED_TITLE, true), |mount| {
        let pressed_button = Button::new("Reveal")
            .on_press(move |runtime| runtime.update(pressed, |presses| presses.push("blurred")))
            .mount(mount);
        button = Some(pressed_button);
        pressed_button
    });
    let button = button.expect("the window mounted its root component");
    stage.application.run_for(SETTLING);

    let content_views = stage
        .native_window(BLURRED_TITLE)
        .and_then(|window| window.contentView());
    let lies_over_blur = content_views.as_ref().is_some_and(|content_view| {
        content_view.downcast_ref::<NSVisualEffectView>().is_some() && content_view.subviews().len() == 1
    });
    let plain_content_is_the_surface = stage
        .native_window(FILES_TITLE)
        .and_then(|window| window.contentView())
        .is_some_and(|content_view| content_view.downcast_ref::<NSVisualEffectView>().is_none());
    stage.expectations.expect(
        lies_over_blur && plain_content_is_the_surface,
        "the surface of a window that asks for blur lies over a view of the system that blurs what is behind it",
        &format!("over blur: {lies_over_blur}, a window that asks for none has none: {plain_content_is_the_surface}"),
    );

    if let Some(events) = stage.events_of(BLURRED_TITLE) {
        events.click(middle_of(&blurred, button));
        stage.application.run_for(ONE_CHANGE);
    }
    stage.expectations.expect(
        stage.presses().last() == Some(&"blurred") && blurred.frame_statistics().presented_frames > 0,
        "the surface over the blur draws and takes the pointer as any other",
        &format!("{:?}, {:?}", stage.presses(), blurred.frame_statistics()),
    );
}

fn the_blur_behind_a_window_shows_in_its_pixels(stage: &mut Stage) {
    let Some(room) = NSScreen::screens(stage.main_thread)
        .firstObject()
        .map(|screen| screen.visibleFrame())
    else {
        return;
    };
    let backdrop = Backdrop::open(
        stage.main_thread,
        NSPoint::new(room.origin.x + BACKDROP_INSET, room.origin.y + BACKDROP_INSET),
    );
    let blurred = stage.open_empty(BLURRED_OVER_EDGE_TITLE, true);
    let plain = stage.open_empty(PLAIN_OVER_EDGE_TITLE, false);
    let (Some(blurred_native), Some(plain_native)) = (
        stage.native_window(BLURRED_OVER_EDGE_TITLE),
        stage.native_window(PLAIN_OVER_EDGE_TITLE),
    ) else {
        return;
    };
    let left = backdrop.edge() - f64::from(OVER_EDGE_SIZE.width) / 2.0;
    blurred_native.setFrameOrigin(NSPoint::new(left, backdrop.bottom() + BLURRED_HEIGHT));
    plain_native.setFrameOrigin(NSPoint::new(left, backdrop.bottom() + PLAIN_HEIGHT));
    blurred_native.makeKeyWindow();
    stage.application.run_for(SETTLING);

    let over_blur = row_across(&blurred_native, backdrop.edge(), stage.main_thread);
    let over_plain = row_across(&plain_native, backdrop.edge(), stage.main_thread);
    let width = over_blur.as_deref().and_then(blur_width);
    stage.expectations.expect(
        width.is_some_and(|width| width >= BLUR_WIDTH),
        "behind a window that asks for blur the edge of red and blue under it shows as a change several pixels wide, its ends leaning to the two colours",
        &format!("{width:?} pixels between the ends of the row {over_blur:?}"),
    );
    stage.expectations.expect(
        over_plain.as_deref().is_some_and(shows_one_colour),
        "a window that asks for no blur shows its own background over the edge",
        &format!("{over_plain:?}"),
    );
    drop((blurred, plain, backdrop));
}

fn main() -> ExitCode {
    let main_thread = MainThreadMarker::new().expect("a test that opens windows runs on the main thread");
    let mut stage = Stage::open(main_thread);
    let came_to_front = support::bring_to_front(main_thread);
    stage.expectations.expect(
        came_to_front,
        "the application comes to the front, where its windows take the keyboard and the blur of the system",
        "the system kept another application in front",
    );
    stage.application.run_for(SETTLING);

    a_registered_shortcut_stands_in_the_menu_bar_with_its_chord(&mut stage);
    the_line_of_the_menu_bar_runs_the_handler_which_opens_a_further_window(&mut stage);
    two_windows_keep_trees_and_focus_of_their_own(&mut stage);
    the_menu_bar_shows_the_menus_of_the_window_that_has_the_keyboard(&mut stage);
    a_handler_closes_the_further_window_and_the_first_goes_on(&mut stage);
    the_frame_closes_a_window_and_the_application_hears_of_it(&mut stage);
    a_window_that_asks_for_it_lies_over_the_blur_of_the_system(&mut stage);
    the_blur_behind_a_window_shows_in_its_pixels(&mut stage);

    stage.expectations.report("multiple windows")
}
