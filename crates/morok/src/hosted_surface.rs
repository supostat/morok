use std::cell::{RefCell, RefMut};
use std::mem;
use std::ops::Range;
use std::rc::Rc;
use std::time::Duration;

use morok_components::view::SurfaceId;
use morok_core::accesskit::{ActionRequest, Node, NodeId, Role, TreeId, TreeInfo, TreeUpdate};
use morok_core::{
    DisplayList, DragOutcome, ElementId, ElementTree, FrameScheduler, KeyEvent, Modifiers, NativeMenu, NativeMenuItem,
    OverlaySide, Payload, PayloadType, Point, PointerInput, Rect, ScrollInput, Size, TextCommand,
};
use morok_platform::{
    FocusedText, OverlayId, OverlayRequest, PasteboardClipboard, SurfaceContent, SurfaceKey, SurfaceSettings,
    SystemDragRequest,
};
use morok_style::ColorScheme;

use crate::application_state::ApplicationState;

/// The node a surface that was closed answers an assistive technology with.
const CLOSED_SURFACE_NODE: NodeId = NodeId(0);

/// What a window of the system shows: one surface of the application, of a window or of an
/// overlay. It gives the surface the input of its window, runs the frames of its tree and
/// tells the window which overlays are opened over the surface.
///
/// The surface of a window is closed when the window lets go of this value. The surface of an
/// overlay is closed by the component that opened it, a frame before its window goes: until
/// then the window is answered as an empty surface would answer it.
#[derive(Debug)]
pub(crate) struct HostedSurface {
    pub(crate) state: Rc<RefCell<ApplicationState>>,
    pub(crate) surface: SurfaceId,
    /// The overlays over the surface the window was told of after the last frame.
    listed_overlays: Vec<SurfaceId>,
    /// Scratch space for the overlays opened over the surface now.
    open_overlays: Vec<SurfaceId>,
    /// The time of the last frame the tree of the surface ran. The windows of an overlay and
    /// of the surface under it have a display link each, which may tell the time of the same
    /// frame a little apart: the time of a tree never goes back.
    last_frame_time: Duration,
    /// The focus of the accessibility tree as the window was told last.
    accessibility_focus: NodeId,
    /// The drag source of the drag the window began in the system and has not told the end of.
    dragged_source: Option<ElementId>,
}

impl HostedSurface {
    /// The content of the window that shows `surface`, whose tree has run no frame later
    /// than `last_frame_time`.
    pub(crate) fn new(state: Rc<RefCell<ApplicationState>>, surface: SurfaceId, last_frame_time: Duration) -> Self {
        Self {
            state,
            surface,
            listed_overlays: Vec::new(),
            open_overlays: Vec::new(),
            last_frame_time,
            accessibility_focus: CLOSED_SURFACE_NODE,
            dragged_source: None,
        }
    }

    /// Input went to the components; when a handler wrote a signal, the effects it woke run
    /// in the frame this asks for.
    fn request_frame_for_queued_effects(state: &mut ApplicationState, surface: SurfaceId) {
        if state.runtime.has_queued_effects() {
            state.surfaces.surface_mut(surface).tree_mut().request_frame();
        }
    }

    fn told_of(&mut self, update: TreeUpdate) -> TreeUpdate {
        self.accessibility_focus = update.focus;
        update
    }
}

impl SurfaceContent for HostedSurface {
    /// The stages run in a fixed order: the events and the timers that wait, the effects,
    /// style, layout and the display list. The window renders what `show` is given and asks
    /// for the update of the accessibility tree after it. An overlay whose layout changed
    /// asks the surface under it for a frame, after which its window is placed again.
    fn frame(&mut self, viewport: Size, scale: f32, now: Duration, show: &mut dyn FnMut(&ElementTree)) {
        let now = now.max(self.last_frame_time);
        self.last_frame_time = now;
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        let state = &mut *state;
        let surface = state.surfaces.surface_mut(self.surface);
        surface.refresh_hover(&mut state.runtime);
        surface.follow_scrolls(&mut state.runtime);
        surface.run_due_timers(&mut state.runtime, now);
        state.flush();
        if !state.surfaces.contains(self.surface) {
            return;
        }
        let tree = state.surfaces.surface_mut(self.surface).tree_mut();
        tree.resolve_styles(now);
        let laid_out_elements = tree.compute_layout(viewport, scale, &mut state.text_shaper);
        tree.build_display_list();
        show(tree);
        if laid_out_elements > 0
            && let Some(parent) = state.surfaces.parent_of(self.surface)
        {
            state.surfaces.surface_mut(parent).tree_mut().request_frame();
        }
    }

    fn color_scheme_changed(&mut self, color_scheme: ColorScheme) {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        let theme = state.styles.theme(color_scheme).clone();
        state.surfaces.surface_mut(self.surface).tree_mut().set_theme(theme);
    }

    fn reduced_motion_changed(&mut self, reduces_motion: bool) {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        state
            .surfaces
            .surface_mut(self.surface)
            .tree_mut()
            .set_reduced_motion(reduces_motion);
    }

    fn pointer_input(&mut self, input: &PointerInput) {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        let state = &mut *state;
        state.surfaces.pointer_input(&mut state.runtime, self.surface, input);
        Self::request_frame_for_queued_effects(state, self.surface);
    }

    /// The key of a surface is the serial of its handle.
    fn surface_key(&self) -> Option<SurfaceKey> {
        Some(SurfaceKey(self.surface.serial()))
    }

    fn pointer_input_over(&mut self, input: &PointerInput, under: SurfaceKey, position: Point) {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        let state = &mut *state;
        let under = SurfaceId::of_serial(under.0);
        if state.surfaces.contains(under) {
            state
                .surfaces
                .pointer_input_over(&mut state.runtime, self.surface, input, under, position);
        } else {
            state.surfaces.pointer_input(&mut state.runtime, self.surface, input);
        }
        Self::request_frame_for_queued_effects(state, self.surface);
    }

    fn scroll_input(&mut self, input: &ScrollInput) {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        state.surfaces.surface_mut(self.surface).scroll_input(input);
    }

    fn key_input(&mut self, event: &KeyEvent) -> bool {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return false;
        };
        let state = &mut *state;
        let was_used = state.surfaces.key_input(&mut state.runtime, self.surface, event);
        Self::request_frame_for_queued_effects(state, self.surface);
        was_used
    }

    fn accessibility_tree(&mut self) -> TreeUpdate {
        let update = match state_while_open(&self.state, self.surface) {
            Some(mut state) => state.surfaces.surface_mut(self.surface).tree_mut().accessibility_tree(),
            None => TreeUpdate {
                nodes: vec![(CLOSED_SURFACE_NODE, Node::new(Role::Window))],
                tree: Some(TreeInfo::new(CLOSED_SURFACE_NODE)),
                tree_id: TreeId::ROOT,
                focus: CLOSED_SURFACE_NODE,
            },
        };
        self.told_of(update)
    }

    fn accessibility_update(&mut self) -> TreeUpdate {
        let update = match state_while_open(&self.state, self.surface) {
            Some(mut state) => state
                .surfaces
                .surface_mut(self.surface)
                .tree_mut()
                .accessibility_update(),
            None => TreeUpdate {
                nodes: Vec::new(),
                tree: None,
                tree_id: TreeId::ROOT,
                focus: self.accessibility_focus,
            },
        };
        self.told_of(update)
    }

    fn accessibility_action(&mut self, request: &ActionRequest) {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        let state = &mut *state;
        state
            .surfaces
            .surface_mut(self.surface)
            .accessibility_action(&mut state.runtime, request);
        Self::request_frame_for_queued_effects(state, self.surface);
    }

    fn focused_text(&mut self) -> Option<FocusedText> {
        let state = state_while_open(&self.state, self.surface)?;
        let tree = state.surfaces.surface(self.surface).tree();
        let focused = tree.focused_text_input()?;
        Some(FocusedText {
            text: tree.text(focused).to_owned(),
            selection: tree.text_selection(focused),
            marked: tree.marked_text(focused),
        })
    }

    fn insert_text(&mut self, text: &str, replacement: Option<Range<usize>>) {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        let state = &mut *state;
        state
            .surfaces
            .surface_mut(self.surface)
            .insert_text(&mut state.runtime, text, replacement);
        Self::request_frame_for_queued_effects(state, self.surface);
    }

    fn set_marked_text(&mut self, text: &str, selected: Range<usize>, replacement: Option<Range<usize>>) {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        let state = &mut *state;
        state
            .surfaces
            .surface_mut(self.surface)
            .set_marked_text(&mut state.runtime, text, selected, replacement);
        Self::request_frame_for_queued_effects(state, self.surface);
    }

    fn unmark_text(&mut self) {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        state.surfaces.surface_mut(self.surface).unmark_text();
    }

    fn text_command(&mut self, command: TextCommand) -> bool {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return false;
        };
        let state = &mut *state;
        let was_taken = state
            .surfaces
            .surface_mut(self.surface)
            .text_command(&mut state.runtime, command);
        Self::request_frame_for_queued_effects(state, self.surface);
        was_taken
    }

    fn text_range_box(&mut self, range: Range<usize>) -> Option<Rect> {
        let state = state_while_open(&self.state, self.surface)?;
        let tree = state.surfaces.surface(self.surface).tree();
        let focused = tree.focused_text_input()?;
        Some(tree.text_range_box(focused, range))
    }

    fn text_offset_at(&mut self, point: Point) -> Option<usize> {
        let state = state_while_open(&self.state, self.surface)?;
        let tree = state.surfaces.surface(self.surface).tree();
        let focused = tree.focused_text_input()?;
        Some(tree.text_offset_at(focused, point))
    }

    /// An overlay the window is told of for the first time is laid out here, in the room it
    /// has: the window has to know how large it is before it shows it.
    fn overlays(&mut self, available: Size, scale: f32, now: Duration, overlays: &mut Vec<OverlayRequest>) {
        let now = now.max(self.last_frame_time);
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        let state = &mut *state;
        state.surfaces.follow_overlay_anchors(&mut state.runtime, self.surface);
        state.surfaces.overlays_of(self.surface, &mut self.open_overlays);
        for overlay in &self.open_overlays {
            if !self.listed_overlays.contains(overlay) {
                let tree = state.surfaces.surface_mut(*overlay).tree_mut();
                tree.resolve_styles(now);
                tree.compute_layout(available, scale, &mut state.text_shaper);
            }
            if let Some(geometry) = state.surfaces.overlay_geometry(*overlay) {
                overlays.push(OverlayRequest {
                    id: OverlayId(overlay.serial()),
                    anchor: geometry.anchor,
                    placement: geometry.placement,
                    size: geometry.size,
                    panel: geometry.panel,
                    input: geometry.input,
                });
            }
        }
        mem::swap(&mut self.listed_overlays, &mut self.open_overlays);
        self.last_frame_time = now;
    }

    /// The tree of the overlay asks the window of the overlay for its frames from here on,
    /// and its texts copy to and paste from the clipboard of the system.
    fn overlay_content(
        &mut self,
        overlay: OverlayId,
        frame_scheduler: Box<dyn FrameScheduler>,
        settings: SurfaceSettings,
    ) -> Rc<RefCell<dyn SurfaceContent>> {
        let surface = *self
            .listed_overlays
            .iter()
            .find(|listed| listed.serial() == overlay.0)
            .expect("a window asks for the content of an overlay it was told of");
        let tree_host = &mut *self.state.borrow_mut();
        let tree = tree_host.surfaces.surface_mut(surface).tree_mut();
        tree.set_reduced_motion(settings.reduces_motion);
        tree.set_frame_scheduler(frame_scheduler);
        tree.set_clipboard(Box::new(PasteboardClipboard::general()));
        Rc::new(RefCell::new(Self::new(
            Rc::clone(&self.state),
            surface,
            self.last_frame_time,
        )))
    }

    fn overlay_placed(&mut self, overlay: OverlayId, side: OverlaySide, origin: Point) {
        let placed = *self
            .listed_overlays
            .iter()
            .find(|listed| listed.serial() == overlay.0)
            .expect("a window places the overlays it was told of");
        let state = &mut *self.state.borrow_mut();
        state.surfaces.overlay_placed(&mut state.runtime, placed, side, origin);
        Self::request_frame_for_queued_effects(state, placed);
    }

    fn dismiss_overlays(&mut self) {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        let state = &mut *state;
        state.surfaces.dismiss_overlays(&mut state.runtime);
        Self::request_frame_for_queued_effects(state, self.surface);
    }

    fn native_menu(&mut self) -> Option<NativeMenu> {
        state_while_open(&self.state, self.surface)?
            .surfaces
            .surface_mut(self.surface)
            .take_native_menu()
    }

    fn native_menu_command_chosen(&mut self, id: usize) {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        let state = &mut *state;
        state
            .surfaces
            .surface_mut(self.surface)
            .native_menu_command_chosen(&mut state.runtime, id);
        Self::request_frame_for_queued_effects(state, self.surface);
    }

    fn menu_bar_revision(&mut self) -> u64 {
        state_while_open(&self.state, self.surface).map_or(0, |state| {
            state.surfaces.surface(self.surface).tree().menu_bar_revision()
        })
    }

    /// The menus are the lines the shortcuts of the tree of the surface gave the menu bar.
    fn menu_bar(&mut self) -> Vec<NativeMenuItem> {
        state_while_open(&self.state, self.surface)
            .map_or_else(Vec::new, |state| state.surfaces.surface(self.surface).tree().menu_bar())
    }

    fn menu_bar_command_chosen(&mut self, id: usize) {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        let state = &mut *state;
        state
            .surfaces
            .surface_mut(self.surface)
            .menu_command_chosen(&mut state.runtime, id);
        Self::request_frame_for_queued_effects(state, self.surface);
    }

    /// The picture of the drag is what the last frame drew of its source.
    fn system_drag(&mut self) -> Option<SystemDragRequest> {
        let mut state = state_while_open(&self.state, self.surface)?;
        let tree = state.surfaces.surface_mut(self.surface).tree_mut();
        let drag = tree.take_system_drag()?;
        let mut picture = DisplayList::default();
        let picture_box = tree.subtree_display_list(drag.source, &mut picture);
        self.dragged_source = Some(drag.source);
        Some(SystemDragRequest {
            payload: drag.payload,
            picture,
            theme: tree.theme().clone(),
            picture_box,
        })
    }

    fn system_drag_ended(&mut self, outcome: DragOutcome) {
        let Some(source) = self.dragged_source.take() else {
            return;
        };
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        let state = &mut *state;
        state
            .surfaces
            .surface_mut(self.surface)
            .system_drag_ended(&mut state.runtime, source, outcome);
        Self::request_frame_for_queued_effects(state, self.surface);
    }

    fn drop_types_revision(&mut self) -> u64 {
        state_while_open(&self.state, self.surface).map_or(0, |state| {
            state.surfaces.surface(self.surface).tree().drop_types_revision()
        })
    }

    /// The types are the ones the drop targets of the tree of the surface take between them.
    fn drop_types(&mut self) -> Vec<PayloadType> {
        state_while_open(&self.state, self.surface).map_or_else(Vec::new, |state| {
            state.surfaces.surface(self.surface).tree().accepted_drop_types()
        })
    }

    fn system_drag_entered(&mut self, payload: Payload, position: Point, modifiers: Modifiers) -> bool {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return false;
        };
        let state = &mut *state;
        let is_taken = state.surfaces.surface_mut(self.surface).system_drag_entered(
            &mut state.runtime,
            payload,
            position,
            modifiers,
        );
        Self::request_frame_for_queued_effects(state, self.surface);
        is_taken
    }

    fn system_drag_moved(&mut self, position: Point, modifiers: Modifiers) -> bool {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return false;
        };
        let state = &mut *state;
        let is_taken =
            state
                .surfaces
                .surface_mut(self.surface)
                .system_drag_moved(&mut state.runtime, position, modifiers);
        Self::request_frame_for_queued_effects(state, self.surface);
        is_taken
    }

    fn system_drag_exited(&mut self) {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return;
        };
        let state = &mut *state;
        state
            .surfaces
            .surface_mut(self.surface)
            .system_drag_exited(&mut state.runtime);
        Self::request_frame_for_queued_effects(state, self.surface);
    }

    fn system_drag_dropped(&mut self, position: Point, modifiers: Modifiers) -> bool {
        let Some(mut state) = state_while_open(&self.state, self.surface) else {
            return false;
        };
        let state = &mut *state;
        let was_taken =
            state
                .surfaces
                .surface_mut(self.surface)
                .system_drag_dropped(&mut state.runtime, position, modifiers);
        Self::request_frame_for_queued_effects(state, self.surface);
        was_taken
    }
}

/// The state of the application, borrowed while `surface` is open.
fn state_while_open(state: &RefCell<ApplicationState>, surface: SurfaceId) -> Option<RefMut<'_, ApplicationState>> {
    let state = state.borrow_mut();
    state.surfaces.contains(surface).then_some(state)
}

impl Drop for HostedSurface {
    /// The surface of a window goes with its window; an overlay is closed by the component
    /// that opened it, or with the surface it is opened over.
    fn drop(&mut self) {
        let state = &mut *self.state.borrow_mut();
        if state.surfaces.contains(self.surface) && state.surfaces.parent_of(self.surface).is_none() {
            state.surfaces.close(&mut state.runtime, self.surface);
        }
    }
}
