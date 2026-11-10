mod support;

use std::process::ExitCode;
use std::time::Duration;

use morok::components::Button;
use morok::core::{ElementId, Point, Size};
use morok::reactive::Signal;
use morok::{Application, Styles, Window, WindowOptions};
use objc2::MainThreadMarker;
use support::{AccessibleElement, EventSource, Expectations};

const CONTENT_SIZE: Size = Size {
    width: 400.0,
    height: 300.0,
};
/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// Long enough for a requested frame to be drawn.
const ONE_FRAME: Duration = Duration::from_millis(500);
/// The virtual key codes of the space bar and of the Return key.
const SPACE_KEY: u16 = 0x31;
const RETURN_KEY: u16 = 0x24;

/// What the root component of the window built.
#[derive(Clone, Copy, Debug)]
struct Built {
    save: ElementId,
    unavailable: ElementId,
    /// The text that shows how often Save was pressed.
    count: ElementId,
}

/// The window of the test with what the expectations read of it.
struct Stage {
    application: Application,
    window: Window,
    source: EventSource,
    expectations: Expectations,
    built: Built,
    save_presses: Signal<u32>,
    unavailable_presses: Signal<u32>,
}

impl Stage {
    /// A window on a button that counts its presses, a text that shows the count, and a
    /// disabled button that counts its presses too.
    fn open(main_thread: MainThreadMarker) -> Self {
        let application = Application::new(Styles::of_kit()).expect("this Mac runs Metal 4");
        let (save_presses, unavailable_presses) =
            application.update(|runtime| (runtime.signal(0_u32), runtime.signal(0_u32)));
        let options = WindowOptions {
            title: String::from("morok: button interaction"),
            content_size: CONTENT_SIZE,
        };
        let mut built = None;
        let window = application.open_window(&options, |mount| {
            let save = Button::new("Save")
                .on_press(move |runtime| runtime.update(save_presses, |presses| *presses += 1))
                .mount(mount);
            let unavailable = Button::new("Unavailable")
                .disabled(true)
                .on_press(move |runtime| runtime.update(unavailable_presses, |presses| *presses += 1))
                .mount(mount);
            let shown_count = mount
                .runtime()
                .memo(move |runtime| runtime.get(save_presses).to_string());
            let count = mount.text(shown_count).id();
            built = Some(Built {
                save,
                unavailable,
                count,
            });
            mount.element().child(save).child(unavailable).child(count).id()
        });
        Self {
            application,
            window,
            source: EventSource::for_the_window(main_thread, CONTENT_SIZE),
            expectations: Expectations::default(),
            built: built.expect("the window mounted its root component"),
            save_presses,
            unavailable_presses,
        }
    }

    fn presses(&self, of: Signal<u32>) -> u32 {
        self.application.update(|runtime| runtime.get(of))
    }

    /// What the text bound to the presses of Save shows.
    fn shown_count(&self) -> String {
        self.window.with_tree(|tree| tree.text(self.built.count).to_owned())
    }

    /// The middle of the box the last frame gave the element.
    fn middle_of(&self, element: ElementId) -> Point {
        let border_box = self.window.with_tree(|tree| tree.border_box(element));
        Point {
            x: border_box.x + border_box.width / 2.0,
            y: border_box.y + border_box.height / 2.0,
        }
    }

    /// The elements of the two buttons as the system sees them, in tree order.
    fn button_elements(&self) -> Vec<AccessibleElement> {
        AccessibleElement::of_view(self.source.surface_view())
            .children()
            .first()
            .map_or_else(Vec::new, AccessibleElement::children)
    }

    fn the_first_frame_shows_the_buttons(&mut self) {
        self.application.run_for(SETTLING);
        let statistics = self.window.frame_statistics();
        self.expectations.expect(
            statistics.presented_frames >= 1,
            "the window presents a first frame; is the display on?",
            &format!("{statistics:?}"),
        );
        let (save, shown) = self
            .window
            .with_tree(|tree| (tree.border_box(self.built.save), tree.text(self.built.count).to_owned()));
        self.expectations.expect(
            save.height.to_bits() == 28.0_f32.to_bits() && shown == "0",
            "the first frame lays the button out by its stylesheet and shows the bound text",
            &format!("{save:?} and {shown:?}"),
        );
    }

    fn a_click_presses_the_button_and_the_next_frame_shows_it(&mut self) {
        self.source.click(self.middle_of(self.built.save));
        let presses = self.presses(self.save_presses);
        self.expectations
            .expect(presses == 1, "a click on the button presses it", &presses.to_string());
        let focused = self.window.with_tree(|tree| tree.focused());
        self.expectations.expect(
            focused == Some(self.built.save),
            "the press gives the button focus",
            &format!("{focused:?}"),
        );

        self.application.run_for(ONE_FRAME);
        let shown = self.shown_count();
        self.expectations.expect(
            shown == "1",
            "the frame after the press runs the effect of the text bound to the count",
            &shown,
        );
    }

    fn space_and_return_press_the_focused_button(&mut self) {
        self.source.press_key(SPACE_KEY, " ");
        let after_space = self.presses(self.save_presses);
        self.expectations.expect(
            after_space == 2,
            "Space presses the button that has focus",
            &after_space.to_string(),
        );
        // The first key shows the focus of the button, which is a change of the tree: the
        // frame it asks for is over before the next key.
        self.application.run_for(ONE_FRAME);

        // Nothing but the key itself asks for the frame that follows it: the signals are
        // read after the frame.
        self.source.press_key(RETURN_KEY, "\r");
        self.application.run_for(ONE_FRAME);
        let shown = self.shown_count();
        self.expectations.expect(
            shown == "3",
            "a key that changes nothing of the tree itself still gets the frame that runs its effects",
            &shown,
        );
        let after_return = self.presses(self.save_presses);
        self.expectations.expect(
            after_return == 3,
            "Return presses the button that has focus",
            &after_return.to_string(),
        );
    }

    fn an_accessibility_action_presses_the_button(&mut self) {
        let buttons = self.button_elements();
        let described: Vec<(Option<String>, Option<String>)> =
            buttons.iter().map(|button| (button.role(), button.title())).collect();
        let expected = ["Save", "Unavailable"].map(|title| (Some(String::from("AXButton")), Some(String::from(title))));
        self.expectations.expect(
            described == expected,
            "the system sees the two buttons with their role and their names",
            &format!("{described:?}"),
        );
        let performed = buttons.first().is_some_and(AccessibleElement::press);
        let presses = self.presses(self.save_presses);
        self.expectations.expect(
            performed && presses == 4,
            "pressing the element of the button through accessibility presses the button",
            &format!("performed: {performed}, presses: {presses}"),
        );
    }

    fn a_disabled_button_is_pressed_by_nothing(&mut self) {
        self.source.click(self.middle_of(self.built.unavailable));
        if let Some(unavailable) = self.button_elements().get(1) {
            unavailable.press();
        }
        let focused = self.window.with_tree(|tree| tree.focused());
        self.source.press_key(SPACE_KEY, " ");
        let (unavailable, save) = (self.presses(self.unavailable_presses), self.presses(self.save_presses));
        self.expectations.expect(
            unavailable == 0,
            "a click and an accessibility action leave the disabled button unpressed",
            &unavailable.to_string(),
        );
        self.expectations.expect(
            focused == Some(self.built.save) && save == 5,
            "the disabled button takes no focus: Space still presses the button that had it",
            &format!("focus on {focused:?}, presses: {save}"),
        );
    }

    fn a_signal_written_from_outside_the_components_shows_in_the_next_frame(&mut self) {
        // No frame is on its way when the signal is written.
        self.application.run_for(ONE_FRAME);
        self.application.update(|runtime| runtime.set(self.save_presses, 40));
        self.application.run_for(ONE_FRAME);
        let shown = self.shown_count();
        self.expectations.expect(
            shown == "40",
            "the application writes a signal and the window shows it after a frame",
            &shown,
        );
    }
}

fn main() -> ExitCode {
    let main_thread = MainThreadMarker::new().expect("the test runs on the main thread");
    let mut stage = Stage::open(main_thread);

    stage.the_first_frame_shows_the_buttons();
    stage.a_click_presses_the_button_and_the_next_frame_shows_it();
    stage.space_and_return_press_the_focused_button();
    stage.an_accessibility_action_presses_the_button();
    stage.a_disabled_button_is_pressed_by_nothing();
    stage.a_signal_written_from_outside_the_components_shows_in_the_next_frame();

    stage.expectations.report("button interaction")
}
