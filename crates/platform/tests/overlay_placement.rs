mod support;

use std::cell::RefCell;
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Duration;

use morok_core::accesskit::{ActionRequest, TreeUpdate};
use morok_core::{
    ElementTree, FrameScheduler, KeyEvent, OverlayAlignment, OverlayPlacement, OverlaySide, Point, PointerInput, Rect,
    ScrollInput, Size,
};
use morok_platform::{Application, OverlayId, OverlayRequest, SurfaceContent, SurfaceSettings, Window, WindowOptions};
use morok_style::ColorScheme;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSWindow};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use support::{Expectations, WindowScene, run_main_loop};

const PAGE: &str = ".page { background-color: var(--color-paper); }";
/// The surface of the overlay: a red panel with 32 px of room around it.
const OVERLAY: &str = "\
    .room { position: absolute; top: 0; left: 0; padding: 32px; } \
    .panel { width: 200px; height: 120px; background-color: var(--color-red); }";

const WINDOW: Size = Size {
    width: 400.0,
    height: 300.0,
};
const OVERLAY_SURFACE: Size = Size {
    width: 264.0,
    height: 184.0,
};
const PANEL: Rect = Rect::new(32.0, 32.0, 200.0, 120.0);
const GAP: f32 = 4.0;
const BELOW_ITS_START: OverlayPlacement = OverlayPlacement {
    side: OverlaySide::Below,
    alignment: OverlayAlignment::Start,
    gap: GAP,
    shift: 0.0,
};
const THE_OVERLAY: OverlayId = OverlayId(7);

/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// Long enough for the system to deliver a change and for the frames it asks for to be drawn.
const ONE_CHANGE: Duration = Duration::from_millis(700);
/// How far a window may lie from where it is expected: windows lie on whole pixels.
const TOLERANCE: f64 = 0.5;
/// How far the corner the content is told of may lie from the expected one: it is worked out
/// from places on the desktop, thousands of points from its origin.
const CORNER_TOLERANCE: f32 = 0.01;

/// A window scene that asks for one overlay, placed against `anchor`, while it has one.
struct AnchoredScene {
    scene: WindowScene,
    anchor: Option<Rect>,
    /// Where the window said it placed the overlay, every time it said so.
    placements: Vec<(OverlaySide, Point)>,
}

impl SurfaceContent for AnchoredScene {
    fn frame(&mut self, viewport: Size, scale: f32, now: Duration, show: &mut dyn FnMut(&ElementTree)) {
        self.scene.frame(viewport, scale, now, show);
    }

    fn color_scheme_changed(&mut self, color_scheme: ColorScheme) {
        self.scene.color_scheme_changed(color_scheme);
    }

    fn reduced_motion_changed(&mut self, reduces_motion: bool) {
        self.scene.reduced_motion_changed(reduces_motion);
    }

    fn pointer_input(&mut self, _input: &PointerInput) {}

    fn scroll_input(&mut self, _input: &ScrollInput) {}

    fn key_input(&mut self, _event: &KeyEvent) -> bool {
        false
    }

    fn accessibility_tree(&mut self) -> TreeUpdate {
        self.scene.accessibility_tree()
    }

    fn accessibility_update(&mut self) -> TreeUpdate {
        self.scene.accessibility_update()
    }

    fn accessibility_action(&mut self, _request: &ActionRequest) {}

    fn overlays(&mut self, _available: Size, _scale: f32, _now: Duration, overlays: &mut Vec<OverlayRequest>) {
        overlays.extend(self.anchor.map(|anchor| OverlayRequest {
            id: THE_OVERLAY,
            anchor,
            placement: BELOW_ITS_START,
            size: OVERLAY_SURFACE,
            panel: PANEL,
            takes_input: true,
        }));
    }

    fn overlay_content(
        &mut self,
        _overlay: OverlayId,
        frame_scheduler: Box<dyn FrameScheduler>,
        settings: SurfaceSettings,
    ) -> Rc<RefCell<dyn SurfaceContent>> {
        let mut content = WindowScene::new(OVERLAY, frame_scheduler, settings);
        let room = content.scene.add(content.scene.root(), &["room"]);
        content.scene.add(room, &["panel"]);
        Rc::new(RefCell::new(content))
    }

    fn overlay_placed(&mut self, _overlay: OverlayId, side: OverlaySide, origin: Point) {
        self.placements.push((side, origin));
    }
}

/// The windows of the application as `AppKit` has them: the window of the test and the
/// window of its overlay, when one is on screen.
struct Windows {
    main_thread: MainThreadMarker,
}

impl Windows {
    fn all(&self) -> Vec<Retained<NSWindow>> {
        NSApplication::sharedApplication(self.main_thread).windows().to_vec()
    }

    fn parent(&self) -> Retained<NSWindow> {
        self.all()
            .into_iter()
            .find(|window| window.parentWindow().is_none() && window.isVisible())
            .expect("the window of the test is on screen")
    }

    fn overlay(&self) -> Option<Retained<NSWindow>> {
        self.all().into_iter().find(|window| window.parentWindow().is_some())
    }

    /// The rectangle of the desktop a rectangle of the surface of the parent covers. `AppKit`
    /// measures the desktop upwards and the surface is measured downwards from its top.
    fn on_desktop(&self, rect: Rect) -> NSRect {
        let parent = self.parent();
        let surface = parent.contentRectForFrameRect(parent.frame());
        let top = surface.origin.y + surface.size.height;
        NSRect::new(
            NSPoint::new(
                surface.origin.x + f64::from(rect.x),
                top - f64::from(rect.y) - f64::from(rect.height),
            ),
            NSSize::new(f64::from(rect.width), f64::from(rect.height)),
        )
    }

    /// Where the panel of the overlay lies on the desktop: the frame of its window without
    /// the room around the panel.
    fn panel_on_desktop(&self) -> Option<NSRect> {
        let frame = self.overlay()?.frame();
        Some(NSRect::new(
            NSPoint::new(
                frame.origin.x + f64::from(PANEL.x),
                frame.origin.y + frame.size.height - f64::from(PANEL.y) - f64::from(PANEL.height),
            ),
            NSSize::new(f64::from(PANEL.width), f64::from(PANEL.height)),
        ))
    }
}

fn is_close(actual: NSRect, expected: NSRect) -> bool {
    (actual.origin.x - expected.origin.x).abs() <= TOLERANCE
        && (actual.origin.y - expected.origin.y).abs() <= TOLERANCE
        && (actual.size.width - expected.size.width).abs() <= TOLERANCE
        && (actual.size.height - expected.size.height).abs() <= TOLERANCE
}

/// Has the scene ask for its overlay against `anchor`, or for none, from its next frame on.
fn anchor_overlay(window: &Window<AnchoredScene>, anchor: Option<Rect>) {
    window.update(|content| {
        content.anchor = anchor;
        content.scene.scene.tree.request_frame();
    });
}

/// The panel as large as it is, under `anchor` and starting where the anchor starts.
fn under(anchor: Rect) -> Rect {
    Rect::new(anchor.x, anchor.bottom() + GAP, PANEL.width, PANEL.height)
}

/// The panel as large as it is, over `anchor` and starting where the anchor starts.
fn over(anchor: Rect) -> Rect {
    Rect::new(anchor.x, anchor.y - GAP - PANEL.height, PANEL.width, PANEL.height)
}

fn an_overlay_lands_under_its_anchor(
    window: &Window<AnchoredScene>,
    windows: &Windows,
    expectations: &mut Expectations,
) {
    let anchor = Rect::new(40.0, 60.0, 80.0, 28.0);

    anchor_overlay(window, Some(anchor));
    run_main_loop(SETTLING);

    let panel = windows.panel_on_desktop();
    let expected = windows.on_desktop(under(anchor));
    expectations.expect(
        panel.is_some_and(|panel| is_close(panel, expected)),
        "the panel of an overlay lies under its anchor, starting where the anchor starts",
        &format!("the panel is at {panel:?}, expected at {expected:?}"),
    );
    let frame = windows.overlay().map(|overlay| overlay.frame().size);
    expectations.expect(
        frame.is_some_and(|frame| {
            (frame.width - f64::from(OVERLAY_SURFACE.width)).abs() <= TOLERANCE
                && (frame.height - f64::from(OVERLAY_SURFACE.height)).abs() <= TOLERANCE
        }),
        "the window of an overlay is as large as the surface of the overlay",
        &format!("{frame:?}"),
    );
    let placements = window.update(|content| content.placements.clone());
    let surface_corner = Point {
        x: anchor.x - PANEL.x,
        y: anchor.bottom() + GAP - PANEL.y,
    };
    let is_told_once = matches!(
        placements.as_slice(),
        [(OverlaySide::Below, corner)]
            if (corner.x - surface_corner.x).abs() <= CORNER_TOLERANCE
                && (corner.y - surface_corner.y).abs() <= CORNER_TOLERANCE
    );
    expectations.expect(
        is_told_once,
        "the content is told once where its overlay lies",
        &format!("{placements:?}, expected below at {surface_corner:?}"),
    );
    let is_child_of_the_window = windows.overlay().is_some_and(|overlay| {
        overlay
            .parentWindow()
            .is_some_and(|parent| parent.windowNumber() == windows.parent().windowNumber())
    });
    expectations.expect(
        is_child_of_the_window,
        "the window of an overlay belongs to the window it is opened over",
        "it has another parent",
    );
}

fn an_overlay_extends_past_the_window_it_is_opened_over(
    window: &Window<AnchoredScene>,
    windows: &Windows,
    expectations: &mut Expectations,
) {
    let in_the_corner = Rect::new(340.0, 200.0, 50.0, 28.0);

    anchor_overlay(window, Some(in_the_corner));
    run_main_loop(ONE_CHANGE);

    let panel = windows.panel_on_desktop();
    let expected = windows.on_desktop(under(in_the_corner));
    let parent = windows.parent().frame();
    expectations.expect(
        panel.is_some_and(|panel| is_close(panel, expected)),
        "an overlay follows its anchor to the corner of the window",
        &format!("the panel is at {panel:?}, expected at {expected:?}"),
    );
    expectations.expect(
        panel.is_some_and(|panel| panel.origin.x + panel.size.width > parent.origin.x + parent.size.width + 100.0),
        "the panel reaches past the right edge of the window it is opened over",
        &format!("the panel is at {panel:?}, the window at {parent:?}"),
    );
}

fn an_overlay_flips_over_its_anchor_at_the_bottom_of_the_screen(
    window: &Window<AnchoredScene>,
    windows: &Windows,
    expectations: &mut Expectations,
) {
    let near_the_bottom = Rect::new(40.0, 250.0, 80.0, 28.0);
    let parent = windows.parent();
    let room = parent
        .screen()
        .expect("the window of the test is on a screen")
        .visibleFrame();
    let placements_before = window.update(|content| content.placements.len());

    // The window stands on the bottom of the room the screen has: under the anchor there are
    // 22 px left for a panel 120 px high.
    parent.setFrameOrigin(NSPoint::new(room.origin.x + 80.0, room.origin.y));
    anchor_overlay(window, Some(near_the_bottom));
    run_main_loop(ONE_CHANGE);

    let panel = windows.panel_on_desktop();
    let expected = windows.on_desktop(over(near_the_bottom));
    expectations.expect(
        panel.is_some_and(|panel| is_close(panel, expected)),
        "an overlay with no room under its anchor lies over it",
        &format!("the panel is at {panel:?}, expected at {expected:?}"),
    );
    expectations.expect(
        panel.is_some_and(|panel| panel.origin.y >= room.origin.y - TOLERANCE),
        "a flipped overlay lies inside the room of the screen",
        &format!("the panel is at {panel:?}, the room is {room:?}"),
    );
    let last_placement = window.update(|content| content.placements[placements_before..].last().copied());
    expectations.expect(
        last_placement.is_some_and(|(side, _)| side == OverlaySide::Above),
        "the content is told that its overlay flipped",
        &format!("{last_placement:?}"),
    );
}

fn a_moved_window_takes_its_overlay_along_and_places_it_again(windows: &Windows, expectations: &mut Expectations) {
    let near_the_bottom = Rect::new(40.0, 250.0, 80.0, 28.0);
    let parent = windows.parent();
    let room = parent
        .screen()
        .expect("the window of the test is on a screen")
        .visibleFrame();

    // Lifted off the bottom, the window leaves room under the anchor again.
    parent.setFrameOrigin(NSPoint::new(room.origin.x + 80.0, room.origin.y + 260.0));
    run_main_loop(ONE_CHANGE);

    let panel = windows.panel_on_desktop();
    let expected = windows.on_desktop(under(near_the_bottom));
    expectations.expect(
        panel.is_some_and(|panel| is_close(panel, expected)),
        "an overlay of a window that moved is placed against its anchor again",
        &format!("the panel is at {panel:?}, expected at {expected:?}"),
    );
}

fn an_overlay_the_content_asks_for_no_longer_leaves_the_screen(
    window: &Window<AnchoredScene>,
    windows: &Windows,
    expectations: &mut Expectations,
) {
    anchor_overlay(window, None);
    run_main_loop(ONE_CHANGE);

    expectations.expect(
        windows.overlay().is_none(),
        "the window of an overlay closes when the content asks for the overlay no longer",
        "it is still on screen",
    );
}

fn main() -> ExitCode {
    let main_thread = MainThreadMarker::new().expect("a test that opens windows runs on the main thread");
    let application = Application::new().expect("this Mac runs Metal 4");
    let options = WindowOptions {
        title: String::from("morok: overlay placement"),
        content_size: WINDOW,
    };
    let window = Window::open(&application, &options, |frame_scheduler, settings| {
        let mut scene = WindowScene::new(PAGE, frame_scheduler, settings);
        let root = scene.scene.root();
        scene.scene.add_classes(root, &["page"]);
        AnchoredScene {
            scene,
            anchor: None,
            placements: Vec::new(),
        }
    });
    let windows = Windows { main_thread };
    let mut expectations = Expectations::default();
    run_main_loop(SETTLING);
    expectations.expect(
        windows.overlay().is_none(),
        "a content that asks for no overlay has none",
        "an overlay window is on screen",
    );

    an_overlay_lands_under_its_anchor(&window, &windows, &mut expectations);
    an_overlay_extends_past_the_window_it_is_opened_over(&window, &windows, &mut expectations);
    an_overlay_flips_over_its_anchor_at_the_bottom_of_the_screen(&window, &windows, &mut expectations);
    a_moved_window_takes_its_overlay_along_and_places_it_again(&windows, &mut expectations);
    an_overlay_the_content_asks_for_no_longer_leaves_the_screen(&window, &windows, &mut expectations);

    expectations.report("overlay placement")
}
