use std::cell::Cell;

use morok_core::{PlacedOverlay, Point, Rect, Size};
use objc2::MainThreadOnly;
use objc2::rc::Retained;
use objc2_app_kit::{NSWindow, NSWindowOrderingMode};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use crate::overlay::overlay_id::OverlayId;
use crate::overlay::overlay_panel::OverlayPanel;
use crate::overlay::overlay_request::OverlayRequest;
use crate::overlay::screen_space::{ScreenSpace, snapped_to_pixels};
use crate::surface_settings::SurfaceSettings;
use crate::surface_view::SurfaceView;

/// An overlay on screen: its window and the view of its surface. Closing follows the value.
pub(crate) struct ShownOverlay {
    id: OverlayId,
    panel: Retained<OverlayPanel>,
    view: Retained<SurfaceView>,
    takes_keyboard: bool,
    /// Whether the window was put on screen: it is, once it has a place.
    is_on_screen: Cell<bool>,
    /// Where the overlay lies, as its content was told last.
    placed: Cell<Option<PlacedOverlay>>,
}

impl ShownOverlay {
    /// Builds the window of an overlay `parent` asked for, with the content `parent` gives
    /// for it. The window comes on screen when it is placed.
    pub(crate) fn open(parent: &SurfaceView, request: &OverlayRequest, room: Size, settings: SurfaceSettings) -> Self {
        let main_thread = parent.mtm();
        let frame = NSRect::new(NSPoint::ZERO, window_size(request.size));
        let view = SurfaceView::new(main_thread, parent.renderer(), frame, settings, Some(room), false);
        let content = parent
            .content()
            .borrow_mut()
            .overlay_content(request.id, view.frame_scheduler(), settings);
        view.set_content(content);
        let panel = OverlayPanel::new(main_thread, frame, request.input);
        panel.setContentView(Some(&view));
        panel.makeFirstResponder(Some(&view));
        Self {
            id: request.id,
            panel,
            view,
            takes_keyboard: request.input.takes_keyboard(),
            is_on_screen: Cell::new(false),
            placed: Cell::new(None),
        }
    }

    pub(crate) fn id(&self) -> OverlayId {
        self.id
    }

    /// Puts the window where the request asks: its panel against the anchor on the surface of
    /// `parent`, flipped where the room of the screen ends and moved along the anchor where
    /// an edge of that room stops it. Returns where the overlay lies — its surface in CSS
    /// pixels of the surface of `parent`, the side of the anchor and how far the edges of the
    /// room moved it along that side — when any of it changed since it was placed last.
    pub(crate) fn place(
        &self,
        parent: &SurfaceView,
        parent_window: &NSWindow,
        request: &OverlayRequest,
        screen: ScreenSpace,
        room: Rect,
    ) -> Option<PlacedOverlay> {
        let scale = parent_window.backingScaleFactor() as f32;
        let anchor = screen.of_view_rect(parent, parent_window, request.anchor);
        let placed = request.placement.place(anchor, request.panel.size(), room);
        let surface_corner = snapped_to_pixels(
            Point {
                x: placed.frame.x - request.panel.x,
                y: placed.frame.y - request.panel.y,
            },
            scale,
        );
        let size = window_size(request.size);
        let frame = screen.to_desktop(Rect {
            x: surface_corner.x,
            y: surface_corner.y,
            width: size.width as f32,
            height: size.height as f32,
        });
        if frame != self.panel.frame() {
            self.panel.setFrame_display(frame, true);
        }
        self.view.set_overlay_room(room.size());
        if !self.is_on_screen.replace(true) {
            // SAFETY: the panel is a window of its own that is no child of another window,
            // and it leaves its parent before it is closed.
            unsafe { parent_window.addChildWindow_ordered(&self.panel, NSWindowOrderingMode::Above) };
            if self.takes_keyboard {
                self.panel.makeKeyWindow();
            }
        }
        let parent_corner = screen.of_view_rect(parent, parent_window, Rect::ZERO).origin();
        let placement = PlacedOverlay {
            frame: Rect {
                x: surface_corner.x - parent_corner.x,
                y: surface_corner.y - parent_corner.y,
                width: request.size.width,
                height: request.size.height,
            },
            side: placed.side,
            edge_shift: placed.edge_shift,
        };
        (self.placed.replace(Some(placement)) != Some(placement)).then_some(placement)
    }
}

impl Drop for ShownOverlay {
    /// The keyboard goes back to the window the overlay was opened over.
    fn drop(&mut self) {
        self.view.close();
        let had_keyboard = self.panel.isKeyWindow();
        let parent_window = self.panel.parentWindow();
        if let Some(parent_window) = &parent_window {
            parent_window.removeChildWindow(&self.panel);
        }
        self.panel.orderOut(None);
        self.panel.close();
        if had_keyboard && let Some(parent_window) = parent_window {
            parent_window.makeKeyWindow();
        }
    }
}

/// The size of the window of a surface: whole points, so that the surface has whole pixels.
fn window_size(surface: Size) -> NSSize {
    NSSize::new(f64::from(surface.width.ceil()), f64::from(surface.height.ceil()))
}
