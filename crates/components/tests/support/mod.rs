#![allow(dead_code)]

use std::time::Duration;

use morok_components::styles::STYLE_SHEET;
use morok_components::tokens::LIGHT;
use morok_components::view::{Mount, SurfaceId, Surfaces};
use morok_core::accesskit::{Action, ActionData, ActionRequest, Node, NodeId, TreeId, TreeUpdate};
use morok_core::{
    CaretLine, CaretLines, CaretStop, ElementId, ElementTree, FontId, FontMetrics, FontRequest, ItemRange, Key,
    KeyEvent, KeyPhase, Modifiers, Point, PointerButton, PointerInput, PointerInputKind, ShapedGlyph, ShapedLine,
    ShapedRun, ShapedText, Size, TextExtent, TextRequest, TextShaper,
};
use morok_reactive::Runtime;

const VIEWPORT: Size = Size {
    width: 400.0,
    height: 300.0,
};
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
        self.flush();
        self.time += Duration::from_millis(16);
        let tree = self.surfaces.surface_mut(self.surface).tree_mut();
        tree.resolve_styles(self.time);
        tree.compute_layout(VIEWPORT, 1.0, &mut self.text_shaper);
        tree.build_display_list();
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
        let input = PointerInput {
            kind,
            position,
            modifiers: Modifiers::NONE,
        };
        self.surfaces
            .surface_mut(self.surface)
            .pointer_input(&mut self.runtime, &input);
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

    /// The middle of the box the last frame gave the element.
    pub(crate) fn middle_of(&self, element: ElementId) -> Point {
        let border_box = self.tree().border_box(element);
        Point {
            x: border_box.x + border_box.width / 2.0,
            y: border_box.y + border_box.height / 2.0,
        }
    }

    /// Clicks in the middle of the box the last frame gave the element.
    pub(crate) fn click(&mut self, element: ElementId) {
        self.click_at(self.middle_of(element));
    }

    /// A key goes down and up with the modifiers held. Returns whether the press was used.
    pub(crate) fn press_key(&mut self, key: Key, modifiers: Modifiers) -> bool {
        let surface = self.surfaces.surface_mut(self.surface);
        let mut event = KeyEvent {
            phase: KeyPhase::Pressed,
            key,
            modifiers,
            is_repeat: false,
        };
        let was_used = surface.key_input(&mut self.runtime, &event);
        event.phase = KeyPhase::Released;
        surface.key_input(&mut self.runtime, &event);
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
        self.surfaces
            .surface_mut(self.surface)
            .key_input(&mut self.runtime, &event)
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
        let id = NodeId::from(element);
        self.accessibility_tree()
            .nodes
            .into_iter()
            .find(|(node_id, _)| *node_id == id)
            .map(|(_, node)| node)
            .expect("the element has a node in the accessibility tree")
    }
}
