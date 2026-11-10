use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;
use std::time::Duration;

use morok_components::view::SurfaceId;
use morok_core::accesskit::{ActionRequest, TreeUpdate};
use morok_core::{ElementTree, KeyEvent, Point, PointerInput, Rect, ScrollInput, Size, TextCommand};
use morok_platform::{FocusedText, SurfaceContent};
use morok_style::ColorScheme;

use crate::application_state::ApplicationState;

/// What a window of an application shows: one surface of the application. It gives the
/// surface the input of its window and runs the frames of its tree; the surface is closed
/// when the window lets go of its content.
#[derive(Debug)]
pub(crate) struct WindowContent {
    pub(crate) state: Rc<RefCell<ApplicationState>>,
    pub(crate) surface: SurfaceId,
}

impl WindowContent {
    /// Input went to the components; when a handler wrote a signal, the effects it woke run
    /// in the frame this asks for.
    fn request_frame_for_queued_effects(state: &mut ApplicationState, surface: SurfaceId) {
        if state.runtime.has_queued_effects() {
            state.surfaces.surface_mut(surface).tree_mut().request_frame();
        }
    }
}

impl SurfaceContent for WindowContent {
    /// The stages run in a fixed order: the events that wait, the effects, style, layout and
    /// the display list. The window renders what `show` is given and asks for the update of
    /// the accessibility tree after it.
    fn frame(&mut self, viewport: Size, scale: f32, now: Duration, show: &mut dyn FnMut(&ElementTree)) {
        let state = &mut *self.state.borrow_mut();
        let surface = state.surfaces.surface_mut(self.surface);
        surface.refresh_hover(&mut state.runtime);
        surface.follow_scrolls(&mut state.runtime);
        state.flush();
        let tree = state.surfaces.surface_mut(self.surface).tree_mut();
        tree.resolve_styles(now);
        tree.compute_layout(viewport, scale, &mut state.text_shaper);
        tree.build_display_list();
        show(tree);
    }

    fn color_scheme_changed(&mut self, color_scheme: ColorScheme) {
        let state = &mut *self.state.borrow_mut();
        let theme = state.styles.theme(color_scheme).clone();
        state.surfaces.surface_mut(self.surface).tree_mut().set_theme(theme);
    }

    fn reduced_motion_changed(&mut self, reduces_motion: bool) {
        self.state
            .borrow_mut()
            .surfaces
            .surface_mut(self.surface)
            .tree_mut()
            .set_reduced_motion(reduces_motion);
    }

    fn pointer_input(&mut self, input: &PointerInput) {
        let state = &mut *self.state.borrow_mut();
        state
            .surfaces
            .surface_mut(self.surface)
            .pointer_input(&mut state.runtime, input);
        Self::request_frame_for_queued_effects(state, self.surface);
    }

    fn scroll_input(&mut self, input: &ScrollInput) {
        let state = &mut *self.state.borrow_mut();
        state.surfaces.surface_mut(self.surface).scroll_input(input);
    }

    fn key_input(&mut self, event: &KeyEvent) -> bool {
        let state = &mut *self.state.borrow_mut();
        let was_used = state
            .surfaces
            .surface_mut(self.surface)
            .key_input(&mut state.runtime, event);
        Self::request_frame_for_queued_effects(state, self.surface);
        was_used
    }

    fn accessibility_tree(&mut self) -> TreeUpdate {
        self.state
            .borrow_mut()
            .surfaces
            .surface_mut(self.surface)
            .tree_mut()
            .accessibility_tree()
    }

    fn accessibility_update(&mut self) -> TreeUpdate {
        self.state
            .borrow_mut()
            .surfaces
            .surface_mut(self.surface)
            .tree_mut()
            .accessibility_update()
    }

    fn accessibility_action(&mut self, request: &ActionRequest) {
        let state = &mut *self.state.borrow_mut();
        state
            .surfaces
            .surface_mut(self.surface)
            .accessibility_action(&mut state.runtime, request);
        Self::request_frame_for_queued_effects(state, self.surface);
    }

    fn focused_text(&mut self) -> Option<FocusedText> {
        let state = self.state.borrow();
        let tree = state.surfaces.surface(self.surface).tree();
        let focused = tree.focused_text_input()?;
        Some(FocusedText {
            text: tree.text(focused).to_owned(),
            selection: tree.text_selection(focused),
            marked: tree.marked_text(focused),
        })
    }

    fn insert_text(&mut self, text: &str, replacement: Option<Range<usize>>) {
        let state = &mut *self.state.borrow_mut();
        state
            .surfaces
            .surface_mut(self.surface)
            .insert_text(&mut state.runtime, text, replacement);
        Self::request_frame_for_queued_effects(state, self.surface);
    }

    fn set_marked_text(&mut self, text: &str, selected: Range<usize>, replacement: Option<Range<usize>>) {
        let state = &mut *self.state.borrow_mut();
        state
            .surfaces
            .surface_mut(self.surface)
            .set_marked_text(&mut state.runtime, text, selected, replacement);
        Self::request_frame_for_queued_effects(state, self.surface);
    }

    fn unmark_text(&mut self) {
        self.state.borrow_mut().surfaces.surface_mut(self.surface).unmark_text();
    }

    fn text_command(&mut self, command: TextCommand) -> bool {
        let state = &mut *self.state.borrow_mut();
        let was_taken = state
            .surfaces
            .surface_mut(self.surface)
            .text_command(&mut state.runtime, command);
        Self::request_frame_for_queued_effects(state, self.surface);
        was_taken
    }

    fn text_range_box(&mut self, range: Range<usize>) -> Option<Rect> {
        let state = self.state.borrow();
        let tree = state.surfaces.surface(self.surface).tree();
        let focused = tree.focused_text_input()?;
        Some(tree.text_range_box(focused, range))
    }

    fn text_offset_at(&mut self, point: Point) -> Option<usize> {
        let state = self.state.borrow();
        let tree = state.surfaces.surface(self.surface).tree();
        let focused = tree.focused_text_input()?;
        Some(tree.text_offset_at(focused, point))
    }
}

impl Drop for WindowContent {
    fn drop(&mut self) {
        let state = &mut *self.state.borrow_mut();
        state.surfaces.close(&mut state.runtime, self.surface);
    }
}
