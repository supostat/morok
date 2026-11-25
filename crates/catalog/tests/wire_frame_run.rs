mod support;

use std::process::ExitCode;
use std::time::Duration;

use morok::core::{ElementId, ElementTree, Point, Rect, Role};
use morok_catalog::{Catalog, Page, WINDOW_SIZE, WINDOW_TITLE};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::NSWindow;
use support::{AccessibleElement, EventSource, Expectations, bring_to_front, visible_windows, window_titled};

/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// Shorter than the time a pointer rests on a control before its tooltip shows.
const SHORTER_THAN_RESTING: Duration = Duration::from_millis(200);
/// Long enough for the rest of the resting time to pass and for the window of the tooltip to
/// open.
const PAST_RESTING: Duration = Duration::from_millis(1200);
/// Long enough for the frames a move asks for to be drawn and for a window to close.
const ONE_CHANGE: Duration = Duration::from_millis(700);
/// The label of the button the tooltip of the overlays page belongs to.
const TOOLTIP_BUTTON: &str = "Recalculate";
const TOOLTIP: &str = "Recalculate the sizes of every folder";
/// How far, in points, an overlay may lie from where its placement puts it.
const PLACEMENT_TOLERANCE: f64 = 1.0;

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

    fn tree<Outcome>(&self, read: impl FnOnce(&ElementTree) -> Outcome) -> Outcome {
        self.catalog.windows()[0].with_tree(|tree| read(tree))
    }

    fn events(&self) -> Option<EventSource> {
        let window = window_titled(self.main_thread, WINDOW_TITLE)?;
        Some(EventSource::for_window(window, WINDOW_SIZE))
    }

    /// The window of an overlay over the window of the catalog, while one is open.
    fn overlay_window(&self) -> Option<Retained<NSWindow>> {
        visible_windows(self.main_thread)
            .into_iter()
            .find(|window| !window.title().to_string().starts_with(WINDOW_TITLE))
    }
}

/// The box the tooltip of the button belongs to: the box around the button.
fn tooltip_area(tree: &ElementTree) -> Option<Rect> {
    let label = descendants(tree, tree.root())
        .into_iter()
        .find(|element| tree.role(*element) == Role::Text && tree.text(*element) == TOOLTIP_BUTTON)?;
    let button = tree.parent(label)?;
    let area = tree.parent(button)?;
    Some(tree.surface_box(area))
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

fn middle(rect: Rect) -> Point {
    Point {
        x: rect.x + rect.width / 2.0,
        y: rect.y + rect.height / 2.0,
    }
}

/// The bubble of the tooltip in the window of the overlay, as an assistive technology finds it
/// by its text.
fn tooltip_bubble(overlay: &NSWindow) -> Option<AccessibleElement> {
    let view = overlay.contentView().expect("an overlay window shows its surface");
    AccessibleElement::of_view(view)
        .descendants()
        .into_iter()
        .find(|element| element.title().as_deref() == Some(TOOLTIP))
}

/// The pointer rests on the button of the tooltip: the hover of the move, the timer of the
/// resting time and the layout of the overlay before its window opens all run in the frames of
/// the application. The pointer leaves, and the window closes.
fn a_tooltip_opens_once_the_pointer_has_rested_and_closes_when_it_leaves(stage: &mut Stage) {
    let (Some(area), Some(events)) = (stage.tree(tooltip_area), stage.events()) else {
        stage.expectations.expect(
            false,
            "the overlays page shows the button of the tooltip in the window",
            TOOLTIP_BUTTON,
        );
        return;
    };

    events.move_to(middle(area));
    stage.settle(SHORTER_THAN_RESTING);
    stage.expectations.expect(
        stage.overlay_window().is_none(),
        "the tooltip waits until the pointer has rested on its button",
        "an overlay window opened at once",
    );

    stage.settle(PAST_RESTING);
    let overlay = stage.overlay_window();
    let bubble = overlay.as_deref().and_then(tooltip_bubble);
    stage.expectations.expect(
        bubble.is_some(),
        "once the pointer has rested, the tooltip opens in a window of its own with its text",
        &format!("overlay window open: {}", overlay.is_some()),
    );
    if let Some(bubble) = bubble {
        let frame = bubble.frame();
        let under_the_area = events.on_screen(Point {
            x: middle(area).x,
            y: area.bottom(),
        });
        let top = frame.origin.y + frame.size.height;
        let middle_x = frame.origin.x + frame.size.width / 2.0;
        stage.expectations.expect(
            frame.size.width > 0.0
                && frame.size.height > 0.0
                && top <= under_the_area.y + PLACEMENT_TOLERANCE
                && (middle_x - under_the_area.x).abs() <= PLACEMENT_TOLERANCE,
            "the bubble of the tooltip takes the size its layout gave it, under the button and centred on it",
            &format!(
                "bubble {frame:?}, the middle of the bottom of the button at ({}, {})",
                under_the_area.x, under_the_area.y
            ),
        );
    }

    events.move_to(Point {
        x: WINDOW_SIZE.width - 8.0,
        y: WINDOW_SIZE.height - 8.0,
    });
    stage.settle(ONE_CHANGE);
    stage.expectations.expect(
        stage.overlay_window().is_none(),
        "the window of the tooltip closes once the pointer leaves the button",
        "an overlay window is still on screen",
    );
}

fn main() -> ExitCode {
    let main_thread = MainThreadMarker::new().expect("a test that opens windows runs on the main thread");
    let catalog = Catalog::launch(Page::Overlays).expect("this Mac runs Metal 4");
    let mut stage = Stage {
        main_thread,
        catalog,
        expectations: Expectations::default(),
    };
    let came_to_front = bring_to_front(main_thread);
    stage.expectations.expect(
        came_to_front,
        "the catalog comes to the front, where its windows take the pointer",
        "the system kept another application in front",
    );
    stage.settle(SETTLING);

    a_tooltip_opens_once_the_pointer_has_rested_and_closes_when_it_leaves(&mut stage);

    stage.expectations.report("wire frame run")
}
