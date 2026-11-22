#![allow(dead_code)]

use std::time::Duration;

use morok_components::styles::STYLE_SHEET;
use morok_components::tokens::LIGHT;
use morok_components::view::{Mount, SurfaceId, Surfaces};
use morok_core::accesskit::{Action, ActionData, ActionRequest, Node, NodeId, TreeId, TreeUpdate};
use morok_core::{
    CaretLine, CaretLines, CaretStop, ElementId, ElementTree, FontId, FontMetrics, FontRequest, ItemRange, Key,
    KeyEvent, KeyPhase, Modifiers, PlacedOverlay, Point, PointerButton, PointerInput, PointerInputKind, Rect,
    ScrollInput, ScrollPhase, ShapedGlyph, ShapedLine, ShapedRun, ShapedText, Size, TextExtent, TextRequest,
    TextShaper,
};
use morok_reactive::Runtime;

const VIEWPORT: Size = Size {
    width: 400.0,
    height: 300.0,
};
/// The screen of the stage: its window lies in the top left corner, and an overlay has the
/// whole screen for room.
pub(crate) const SCREEN: Rect = Rect::new(0.0, 0.0, 800.0, 600.0);
const FRAME_INTERVAL: Duration = Duration::from_millis(16);
/// The width of every character of the test font, per CSS pixel of font size.
const ADVANCE_PER_FONT_SIZE: f32 = 0.5;

/// Sets every text on one line of characters that are all as wide as each other.
#[derive(Debug, Default)]
struct OneLineShaper;

impl OneLineShaper {
    fn width(request: &TextRequest<'_>) -> f32 {
        request.text.chars().count() as f32 * request.font.size * ADVANCE_PER_FONT_SIZE
    }
}

impl TextShaper for OneLineShaper {
    fn font_metrics(&mut self, font: &FontRequest<'_>) -> FontMetrics {
        FontMetrics {
            ascent: font.size * 0.75,
            descent: font.size * 0.25,
            line_gap: 0.0,
            underline_top: font.size * 0.125,
            line_through_top: font.size * -0.25,
            decoration_thickness: font.size * 0.0625,
        }
    }

    fn measure(&mut self, request: &TextRequest<'_>) -> TextExtent {
        TextExtent {
            width: Self::width(request),
            line_count: usize::from(!request.text.is_empty()),
        }
    }

    fn shape(&mut self, request: &TextRequest<'_>, _ellipsis_width: Option<f32>, shaped: &mut ShapedText) {
        shaped.clear();
        if request.text.is_empty() {
            return;
        }
        let advance = request.font.size * ADVANCE_PER_FONT_SIZE;
        shaped.glyphs.extend(
            request
                .text
                .chars()
                .enumerate()
                .map(|(position, character)| ShapedGlyph {
                    glyph: character as u16,
                    x: position as f32 * advance,
                    y: 0.0,
                }),
        );
        shaped.runs.push(ShapedRun {
            font: FontId(0),
            glyphs: ItemRange {
                start: 0,
                end: shaped.glyphs.len() as u32,
            },
        });
        shaped.lines.push(ShapedLine {
            width: Self::width(request),
            runs: ItemRange { start: 0, end: 1 },
        });
    }

    /// An edited text is one line too, a stop before every character and one after the last.
    fn shape_editable(&mut self, request: &TextRequest<'_>, shaped: &mut ShapedText, carets: &mut CaretLines) {
        self.shape(request, None, shaped);
        carets.clear();
        if request.text.is_empty() {
            return;
        }
        let advance = request.font.size * ADVANCE_PER_FONT_SIZE;
        let offsets = request.text.char_indices().map(|(offset, _)| offset);
        carets.stops.extend(
            offsets
                .chain([request.text.len()])
                .enumerate()
                .map(|(position, offset)| CaretStop {
                    offset: offset as u32,
                    x: position as f32 * advance,
                }),
        );
        carets.lines.push(CaretLine {
            text: ItemRange {
                start: 0,
                end: request.text.len() as u32,
            },
            stops: ItemRange {
                start: 0,
                end: carets.stops.len() as u32,
            },
        });
    }
}

/// One surface of the kit with the signal graph its components live in, driven as a host
/// drives it: input goes to the surface, and a frame flushes the effects and runs the stages
/// of the tree.
pub(crate) struct Stage {
    pub(crate) runtime: Runtime<Surfaces>,
    pub(crate) surfaces: Surfaces,
    pub(crate) surface: SurfaceId,
    text_shaper: OneLineShaper,
    time: Duration,
}

impl Stage {
    /// A surface with an empty tree under the style sheet and the light theme of the kit.
    pub(crate) fn new() -> Self {
        let mut runtime = Runtime::new();
        let mut surfaces = Surfaces::new();
        let surface = surfaces.open(&mut runtime, STYLE_SHEET, LIGHT);
        Self {
            runtime,
            surfaces,
            surface,
            text_shaper: OneLineShaper,
            time: Duration::ZERO,
        }
    }

    /// Opens another surface on the signal graph of the stage.
    pub(crate) fn open_surface(&mut self) -> SurfaceId {
        self.surfaces.open(&mut self.runtime, STYLE_SHEET, LIGHT)
    }

    pub(crate) fn tree(&self) -> &ElementTree {
        self.surfaces.surface(self.surface).tree()
    }

    pub(crate) fn tree_of(&self, surface: SurfaceId) -> &ElementTree {
        self.surfaces.surface(surface).tree()
    }

    pub(crate) fn root(&self) -> ElementId {
        self.tree().root()
    }

    /// Mounts the component under the root of the surface.
    pub(crate) fn mount(&mut self, component: impl FnOnce(&mut Mount<'_>) -> ElementId) -> ElementId {
        self.mount_on(self.surface, component)
    }

    pub(crate) fn mount_on(
        &mut self,
        surface: SurfaceId,
        component: impl FnOnce(&mut Mount<'_>) -> ElementId,
    ) -> ElementId {
        let root = self.tree_of(surface).root();
        self.surfaces.mount(&mut self.runtime, surface, root, component)
    }

    /// Runs the effects that wait.
    pub(crate) fn flush(&mut self) {
        self.runtime
            .flush(&mut self.surfaces)
            .expect("the components of the test have no cycle");
    }

    /// Runs one frame of the surface: the effects that wait, then style, layout and display list.
    pub(crate) fn frame(&mut self) {
        self.time += FRAME_INTERVAL;
        self.frame_of(self.surface, VIEWPORT);
    }

    /// Runs one frame of every surface `after` the frame before, as the windows of a host
    /// do: the surface of the window, then every overlay opened over it, each laid out in
    /// the room of the screen and placed on the screen against its anchor.
    pub(crate) fn frame_all_after(&mut self, after: Duration) {
        self.time += after;
        self.frame_of(self.surface, VIEWPORT);
        self.frame_overlays_of(self.surface, Point::default());
    }

    /// Runs one frame of every surface as [`frame_all`](Self::frame_all) does, with the window
    /// of the stage at `corner` of the screen instead of its top left corner.
    pub(crate) fn frame_all_with_window_at(&mut self, corner: Point) {
        self.time += FRAME_INTERVAL;
        self.frame_of(self.surface, VIEWPORT);
        self.frame_overlays_of(self.surface, corner);
    }

    /// The wheel scrolls by `delta` over a point of a surface, as the window of that surface
    /// reports it.
    pub(crate) fn wheel_on(&mut self, surface: SurfaceId, position: Point, delta: Point) {
        self.surfaces.scroll_input(
            surface,
            &ScrollInput {
                position,
                delta,
                phase: ScrollPhase::Discrete,
            },
        );
    }

    /// Runs one frame of every surface, a frame interval after the frame before.
    pub(crate) fn frame_all(&mut self) {
        self.frame_all_after(FRAME_INTERVAL);
    }

    /// Runs one frame of another surface the stage opened, as the window that shows it would.
    pub(crate) fn frame_surface(&mut self, surface: SurfaceId) {
        self.frame_of(surface, VIEWPORT);
    }

    /// Where the stage placed the top left corner of an overlay opened over its surface, in
    /// CSS pixels of that surface.
    pub(crate) fn origin_of(&self, overlay: SurfaceId) -> Point {
        self.placement_of(overlay, Point::default())
            .expect("the overlay has an anchor to be placed against")
            .frame
            .origin()
    }

    /// Where a window would place the overlay whose parent has its top left corner at
    /// `parent_corner` of the screen: the surface of the overlay on the screen, the side of
    /// its anchor and how far the edges of the screen moved it along that side. None while
    /// its anchor has no box.
    fn placement_of(&self, overlay: SurfaceId, parent_corner: Point) -> Option<PlacedOverlay> {
        let geometry = self.surfaces.overlay_geometry(overlay)?;
        let anchor_on_screen = Rect {
            x: geometry.anchor.x + parent_corner.x,
            y: geometry.anchor.y + parent_corner.y,
            ..geometry.anchor
        };
        let placed = geometry
            .placement
            .place(anchor_on_screen, geometry.panel.size(), SCREEN);
        Some(PlacedOverlay {
            frame: Rect {
                x: placed.frame.x - geometry.panel.x,
                y: placed.frame.y - geometry.panel.y,
                width: geometry.size.width,
                height: geometry.size.height,
            },
            ..placed
        })
    }

    /// Runs the frame of a surface that is open at the time of the stage.
    fn frame_of(&mut self, surface: SurfaceId, viewport: Size) {
        if !self.surfaces.contains(surface) {
            return;
        }
        let shown = self.surfaces.surface_mut(surface);
        shown.follow_scrolls(&mut self.runtime);
        shown.run_due_timers(&mut self.runtime, self.time);
        self.flush();
        if !self.surfaces.contains(surface) {
            return;
        }
        let tree = self.surfaces.surface_mut(surface).tree_mut();
        tree.resolve_styles(self.time);
        tree.compute_layout(viewport, 1.0, &mut self.text_shaper);
        tree.build_display_list();
    }

    /// Runs the frames of the overlays opened over `parent`, whose top left corner lies at
    /// `parent_corner` of the screen, and places each as a window would.
    fn frame_overlays_of(&mut self, parent: SurfaceId, parent_corner: Point) {
        if !self.surfaces.contains(parent) {
            return;
        }
        self.surfaces.follow_overlay_anchors(&mut self.runtime, parent);
        for overlay in self.overlays_of(parent) {
            self.frame_of(overlay, SCREEN.size());
            if !self.surfaces.contains(overlay) {
                continue;
            }
            let Some(placed) = self.placement_of(overlay, parent_corner) else {
                continue;
            };
            let corner = placed.frame.origin();
            let on_parent = PlacedOverlay {
                frame: Rect {
                    x: corner.x - parent_corner.x,
                    y: corner.y - parent_corner.y,
                    ..placed.frame
                },
                ..placed
            };
            self.surfaces.overlay_placed(&mut self.runtime, overlay, on_parent);
            self.frame_overlays_of(overlay, corner);
        }
    }

    /// The overlays opened over the surface.
    pub(crate) fn overlays_of(&self, surface: SurfaceId) -> Vec<SurfaceId> {
        let mut overlays = Vec::new();
        self.surfaces.overlays_of(surface, &mut overlays);
        overlays
    }

    /// The children of the element in the tree of the surface.
    pub(crate) fn children(&self, element: ElementId) -> Vec<ElementId> {
        self.tree().children(element).to_vec()
    }

    /// The texts of the children of the element, in their order.
    pub(crate) fn texts_of_children(&self, element: ElementId) -> Vec<String> {
        self.tree()
            .children(element)
            .iter()
            .map(|child| self.tree().text(*child).to_owned())
            .collect()
    }

    fn pointer(&mut self, kind: PointerInputKind, position: Point) {
        self.pointer_on(self.surface, kind, position);
    }

    /// The pointer does something at a point of a surface, as the window of that surface
    /// reports it.
    pub(crate) fn pointer_on(&mut self, surface: SurfaceId, kind: PointerInputKind, position: Point) {
        let input = PointerInput {
            kind,
            position,
            modifiers: Modifiers::NONE,
        };
        self.surfaces.pointer_input(&mut self.runtime, surface, &input);
    }

    /// What the pointer did while a press on `surface` holds it and it is over `under`, at
    /// `position` of `surface` and at `under_position` of `under`, as the host of `surface`
    /// tells it.
    pub(crate) fn pointer_over(
        &mut self,
        surface: SurfaceId,
        kind: PointerInputKind,
        position: Point,
        under: SurfaceId,
        under_position: Point,
    ) {
        let input = PointerInput {
            kind,
            position,
            modifiers: Modifiers::NONE,
        };
        self.surfaces
            .pointer_input_over(&mut self.runtime, surface, &input, under, under_position);
    }

    /// Presses and releases a button at a point of a surface.
    pub(crate) fn click_on(&mut self, surface: SurfaceId, button: PointerButton, position: Point) {
        self.pointer_on(surface, PointerInputKind::Pressed { button, click_count: 1 }, position);
        self.pointer_on(surface, PointerInputKind::Released { button, click_count: 1 }, position);
    }

    /// The middle of the box a surface showed the element in after its last frame.
    pub(crate) fn middle_on(&self, surface: SurfaceId, element: ElementId) -> Point {
        let shown_in = self.tree_of(surface).surface_box(element);
        Point {
            x: shown_in.x + shown_in.width / 2.0,
            y: shown_in.y + shown_in.height / 2.0,
        }
    }

    /// A key goes down and up on the surface that has the keyboard. Returns whether the
    /// press was used.
    pub(crate) fn press_key_on(&mut self, surface: SurfaceId, key: Key) -> bool {
        let mut event = KeyEvent {
            phase: KeyPhase::Pressed,
            key,
            modifiers: Modifiers::NONE,
            is_repeat: false,
        };
        let was_used = self.surfaces.key_input(&mut self.runtime, surface, &event);
        event.phase = KeyPhase::Released;
        if self.surfaces.contains(surface) {
            self.surfaces.key_input(&mut self.runtime, surface, &event);
        }
        was_used
    }

    /// Presses and releases the primary button at a point of the surface.
    pub(crate) fn click_at(&mut self, position: Point) {
        let button = PointerButton::Primary;
        self.pointer(PointerInputKind::Pressed { button, click_count: 1 }, position);
        self.pointer(PointerInputKind::Released { button, click_count: 1 }, position);
    }

    /// Presses the primary button at `from`, moves the pointer through `through` with the
    /// button down and releases it at the last of them.
    pub(crate) fn drag(&mut self, from: Point, through: &[Point]) {
        let button = PointerButton::Primary;
        self.pointer(PointerInputKind::Pressed { button, click_count: 1 }, from);
        let mut last = from;
        for position in through {
            self.pointer(PointerInputKind::Moved, *position);
            last = *position;
        }
        self.pointer(PointerInputKind::Released { button, click_count: 1 }, last);
    }

    /// Moves the pointer to a point of the surface with no button down.
    pub(crate) fn move_pointer(&mut self, position: Point) {
        self.pointer(PointerInputKind::Moved, position);
    }

    /// The point `fraction` of the way across the box the last frame gave the element, at
    /// half its height.
    pub(crate) fn across(&self, element: ElementId, fraction: f32) -> Point {
        let border_box = self.tree().border_box(element);
        Point {
            x: border_box.x + border_box.width * fraction,
            y: border_box.y + border_box.height / 2.0,
        }
    }

    /// The middle of the box the surface showed the element in after the last frame.
    pub(crate) fn middle_of(&self, element: ElementId) -> Point {
        let border_box = self.tree().surface_box(element);
        Point {
            x: border_box.x + border_box.width / 2.0,
            y: border_box.y + border_box.height / 2.0,
        }
    }

    /// Clicks in the middle of the box the surface showed the element in after the last frame.
    pub(crate) fn click(&mut self, element: ElementId) {
        self.click_at(self.middle_of(element));
    }

    /// A key goes down and up with the modifiers held. Returns whether the press was used.
    pub(crate) fn press_key(&mut self, key: Key, modifiers: Modifiers) -> bool {
        let mut event = KeyEvent {
            phase: KeyPhase::Pressed,
            key,
            modifiers,
            is_repeat: false,
        };
        let was_used = self.surfaces.key_input(&mut self.runtime, self.surface, &event);
        event.phase = KeyPhase::Released;
        self.surfaces.key_input(&mut self.runtime, self.surface, &event);
        was_used
    }

    /// The system repeats a key that is held down.
    pub(crate) fn repeat_key(&mut self, key: Key) -> bool {
        let event = KeyEvent {
            phase: KeyPhase::Pressed,
            key,
            modifiers: Modifiers::NONE,
            is_repeat: true,
        };
        self.surfaces.key_input(&mut self.runtime, self.surface, &event)
    }

    /// Gives keyboard focus to the element. Returns whether it has focus now.
    pub(crate) fn focus(&mut self, element: ElementId) -> bool {
        self.surfaces.surface_mut(self.surface).tree_mut().focus(element)
    }

    /// An assistive technology asks the node of the element for its default action.
    pub(crate) fn accessibility_click(&mut self, element: ElementId) {
        self.accessibility_action(element, Action::Click, None);
    }

    /// An assistive technology asks the node of the element for the action, with its data.
    pub(crate) fn accessibility_action(&mut self, element: ElementId, action: Action, data: Option<ActionData>) {
        let request = ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: NodeId::from(element),
            data,
        };
        self.surfaces
            .surface_mut(self.surface)
            .accessibility_action(&mut self.runtime, &request);
    }

    pub(crate) fn accessibility_tree(&mut self) -> TreeUpdate {
        self.surfaces.surface_mut(self.surface).tree_mut().accessibility_tree()
    }

    /// The node of the element in the accessibility tree as it stands now.
    pub(crate) fn accessible_node(&mut self, element: ElementId) -> Node {
        self.accessible_node_on(self.surface, element)
    }

    /// The node of an element of a surface in the accessibility tree of that surface.
    pub(crate) fn accessible_node_on(&mut self, surface: SurfaceId, element: ElementId) -> Node {
        let id = NodeId::from(element);
        self.surfaces
            .surface_mut(surface)
            .tree_mut()
            .accessibility_tree()
            .nodes
            .into_iter()
            .find(|(node_id, _)| *node_id == id)
            .map(|(_, node)| node)
            .expect("the element has a node in the accessibility tree")
    }
}
