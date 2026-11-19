use morok_core::{Point, Rect};
use objc2::MainThreadMarker;
use objc2_app_kit::{NSScreen, NSView, NSWindow};
use objc2_foundation::{NSPoint, NSRect, NSSize};

/// The coordinates overlays are placed in: points of the desktop measured from the top left
/// corner of the main display, `y` growing downwards as on a surface. `AppKit` measures the
/// desktop from the bottom left corner of that display, upwards.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ScreenSpace {
    /// The height of the main display, whose bottom edge `AppKit` measures from.
    main_display_height: f64,
}

impl ScreenSpace {
    pub(crate) fn current(main_thread: MainThreadMarker) -> Self {
        let main_display_height = NSScreen::screens(main_thread)
            .firstObject()
            .map_or(0.0, |main_display| main_display.frame().size.height);
        Self { main_display_height }
    }

    /// A rectangle of a view, given in CSS pixels of its surface, as it lies on the desktop.
    pub(crate) fn of_view_rect(self, view: &NSView, window: &NSWindow, rect: Rect) -> Rect {
        let in_view = NSRect::new(
            NSPoint::new(f64::from(rect.x), f64::from(rect.y)),
            NSSize::new(f64::from(rect.width), f64::from(rect.height)),
        );
        self.of_desktop_rect(window.convertRectToScreen(view.convertRect_toView(in_view, None)))
    }

    /// The part of the screen a window is on that neither the menu bar nor the Dock covers:
    /// the room an overlay of the window has.
    pub(crate) fn room_around(self, window: &NSWindow, main_thread: MainThreadMarker) -> Rect {
        let screen = window.screen().or_else(|| NSScreen::mainScreen(main_thread));
        screen.map_or(Rect::ZERO, |screen| self.of_desktop_rect(screen.visibleFrame()))
    }

    /// The rectangle as `AppKit` places a window.
    pub(crate) fn to_desktop(self, rect: Rect) -> NSRect {
        NSRect::new(
            NSPoint::new(
                f64::from(rect.x),
                self.main_display_height - f64::from(rect.y) - f64::from(rect.height),
            ),
            NSSize::new(f64::from(rect.width), f64::from(rect.height)),
        )
    }

    fn of_desktop_rect(self, rect: NSRect) -> Rect {
        Rect {
            x: rect.origin.x as f32,
            y: (self.main_display_height - rect.origin.y - rect.size.height) as f32,
            width: rect.size.width as f32,
            height: rect.size.height as f32,
        }
    }
}

/// The point moved to the nearest device pixel of a screen with `scale` pixels to a point.
pub(crate) fn snapped_to_pixels(point: Point, scale: f32) -> Point {
    Point {
        x: (point.x * scale).round() / scale,
        y: (point.y * scale).round() / scale,
    }
}
