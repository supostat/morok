mod support;

use std::cell::RefCell;
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Duration;

use morok::components::Button;
use morok::core::{ElementId, Key, KeyChord, Modifiers, Point, Size};
use morok::reactive::Signal;
use morok::{Application, Styles, Window, WindowOptions};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSEventModifierFlags, NSMenu, NSVisualEffectView, NSWindow};
use support::{EventSource, Expectations};

const CONTENT_SIZE: Size = Size {
    width: 400.0,
    height: 300.0,
};
const FILES_TITLE: &str = "morok: files";
const INSPECTOR_TITLE: &str = "morok: inspector";
const BLURRED_TITLE: &str = "morok: blurred";
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
    /// command that opens the inspector, in the menu "File" of the menu bar.
    fn open(main_thread: MainThreadMarker) -> Self {
        let application = Application::new(Styles::of_kit()).expect("this Mac runs Metal 4");
        let presses = application.update(|runtime| runtime.signal(Vec::new()));
        let opened: Rc<RefCell<Vec<(Window, ElementId)>>> = Rc::default();

        let open_inspector = {
            let (application, opened) = (application.clone(), Rc::clone(&opened));
            move || {
                let mut inspect = None;
                let window = application.open_window(&options(INSPECTOR_TITLE, false), |mount| {
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
}

fn middle_of(window: &Window, element: ElementId) -> Point {
    let shown_in = window.with_tree(|tree| tree.border_box(element));
    Point {
        x: shown_in.x + shown_in.width / 2.0,
        y: shown_in.y + shown_in.height / 2.0,
    }
}

fn a_registered_shortcut_stands_in_the_menu_bar_with_its_chord(stage: &mut Stage) {
    let bar = NSApplication::sharedApplication(stage.main_thread).mainMenu();
    let titles: Vec<String> = bar.as_ref().map_or_else(Vec::new, |bar| {
        (0..bar.numberOfItems())
            .filter_map(|position| bar.itemAtIndex(position))
            .map(|item| item.title().to_string())
            .collect()
    });
    stage.expectations.expect(
        titles.len() == 2 && titles[1] == "File",
        "the menu bar holds the menu of the application and the menu the shortcut names",
        &format!("{titles:?}"),
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

fn main() -> ExitCode {
    let main_thread = MainThreadMarker::new().expect("a test that opens windows runs on the main thread");
    let mut stage = Stage::open(main_thread);
    stage.application.run_for(SETTLING);

    a_registered_shortcut_stands_in_the_menu_bar_with_its_chord(&mut stage);
    the_line_of_the_menu_bar_runs_the_handler_which_opens_a_further_window(&mut stage);
    two_windows_keep_trees_and_focus_of_their_own(&mut stage);
    a_handler_closes_the_further_window_and_the_first_goes_on(&mut stage);
    a_window_that_asks_for_it_lies_over_the_blur_of_the_system(&mut stage);

    stage.expectations.report("multiple windows")
}
