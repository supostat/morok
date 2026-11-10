use std::ops::Range;

use accesskit::{Action, ActionData, ActionRequest};

use super::ElementTree;
use crate::dirty::Dirty;
use crate::editing::clipboard::Clipboard;
use crate::editing::editable_text::{EditChange, EditableText};
use crate::editing::key_bindings::edit_command;
use crate::editing::text_command::TextCommand;
use crate::editing::text_runs::TextRuns;
use crate::editing::text_selection::TextSelection;
use crate::element_id::ElementId;
use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::geometry::transform::Transform;
use crate::input::input_handler::InputHandler;
use crate::input::key_event::KeyEvent;
use crate::input::modifiers::Modifiers;
use crate::input::pointer_button::PointerButton;
use crate::input::pointer_input::PointerInput;
use crate::input::pointer_input_kind::PointerInputKind;
use crate::role::Role;

/// The text an element of an editable role holds: [`Role::TextInput`] and [`Role::TextArea`].
/// Places in the text are bytes and lie between grapheme clusters; a place given inside a
/// cluster falls back on its start.
impl ElementTree {
    /// Names the clipboard the edited texts of the tree copy to and paste from. Without one
    /// the keys of the clipboard go on to the elements around the text.
    pub fn set_clipboard(&mut self, clipboard: Box<dyn Clipboard>) {
        self.clipboard = Some(clipboard);
    }

    /// The text the element shows while it holds none.
    ///
    /// # Panics
    ///
    /// When the element edits no text.
    pub fn placeholder(&self, element: ElementId) -> &str {
        &self.editable(element).placeholder
    }

    /// # Panics
    ///
    /// When the element edits no text.
    pub fn set_placeholder(&mut self, element: ElementId, placeholder: &str) {
        let editable = self.editable_mut(element);
        if editable.placeholder == placeholder {
            return;
        }
        editable.placeholder.clear();
        editable.placeholder.push_str(placeholder);
        self.elements.invalidate_text(element);
        self.text_edited(element, EditChange::View);
    }

    /// # Panics
    ///
    /// When the element edits no text.
    pub fn text_selection(&self, element: ElementId) -> TextSelection {
        self.editable(element).editor.selection()
    }

    /// Moves the selection of the element and scrolls its head into view.
    ///
    /// # Panics
    ///
    /// When the element edits no text.
    pub fn set_text_selection(&mut self, element: ElementId, selection: TextSelection) {
        let change = self.editable_mut(element).select(selection);
        self.text_edited(element, change);
    }

    /// The bytes of the text an input method is composing; none while it composes nothing.
    ///
    /// # Panics
    ///
    /// When the element edits no text.
    pub fn marked_text(&self, element: ElementId) -> Option<Range<usize>> {
        self.editable(element).editor.marked()
    }

    /// How far the lines of the text are scrolled left and up inside the content box of the
    /// element, as the last layout stage left them.
    ///
    /// # Panics
    ///
    /// When the element edits no text.
    pub fn text_scroll(&self, element: ElementId) -> Point {
        self.editable(element).scroll
    }

    /// The focused element when it edits text: where what the user types goes.
    pub fn focused_text_input(&self) -> Option<ElementId> {
        self.input
            .focused
            .filter(|focused| self.elements[*focused].editable.is_some())
    }

    /// Inserts what the user typed or dictated, or what an input method settled on: in place
    /// of the text being composed when there is one, else of `replacement`, else of the
    /// selection. The element hears of the change through `handler`.
    ///
    /// # Panics
    ///
    /// When the element edits no text.
    pub fn insert_text(
        &mut self,
        element: ElementId,
        text: &str,
        replacement: Option<Range<usize>>,
        handler: &mut dyn InputHandler,
    ) {
        let change = self.editable_mut(element).insert(text, replacement);
        self.text_edited_by_user(element, change, handler);
    }

    /// Shows what an input method composes: `text` stands marked in place of the text marked
    /// before, else of `replacement`, else of the selection, with the bytes `selected` of it
    /// selected. An empty text gives the composition up.
    ///
    /// # Panics
    ///
    /// When the element edits no text.
    pub fn set_marked_text(
        &mut self,
        element: ElementId,
        text: &str,
        selected: Range<usize>,
        replacement: Option<Range<usize>>,
        handler: &mut dyn InputHandler,
    ) {
        let change = self.editable_mut(element).set_marked_text(text, selected, replacement);
        self.text_edited_by_user(element, change, handler);
    }

    /// Settles the text an input method composes as it stands.
    ///
    /// # Panics
    ///
    /// When the element edits no text.
    pub fn unmark_text(&mut self, element: ElementId) {
        let change = self.editable_mut(element).unmark();
        self.text_edited(element, change);
    }

    /// The box of the first line of a range of the text as the surface shows it: where an
    /// input method puts its window of candidates.
    ///
    /// # Panics
    ///
    /// When the element edits no text.
    pub fn text_range_box(&self, element: ElementId, range: Range<usize>) -> Rect {
        let in_border_box = self.editable(element).range_rect(range);
        let border_box = self.elements[element].border_box;
        self.surface_transform(element).bounds_of(&Rect::new(
            border_box.x + in_border_box.x,
            border_box.y + in_border_box.y,
            in_border_box.width,
            in_border_box.height,
        ))
    }

    /// The place of the text nearest to a point of the surface.
    ///
    /// # Panics
    ///
    /// When the element edits no text.
    pub fn text_offset_at(&self, element: ElementId, point: Point) -> usize {
        self.editable(element)
            .offset_at(self.point_in_border_box(element, point))
    }

    /// Does a command of the Edit menu to the focused text. Returns whether a text took it:
    /// there is none without focus on an edited text, and the commands of the clipboard need
    /// a clipboard.
    pub fn perform_text_command(&mut self, command: TextCommand, handler: &mut dyn InputHandler) -> bool {
        let Some(focused) = self.focused_text_input() else {
            return false;
        };
        let editable = self.elements[focused]
            .editable
            .as_deref_mut()
            .expect("the focused text input edits text");
        let Some(change) = editable.perform(command.into(), self.clipboard.as_deref_mut()) else {
            return false;
        };
        self.text_edited_by_user(focused, change, handler);
        true
    }

    /// Lets the text of the focused element take a key. Returns whether it did.
    pub(super) fn edit_text_by_key(
        &mut self,
        focused: ElementId,
        event: &KeyEvent,
        handler: &mut dyn InputHandler,
    ) -> bool {
        let Some(editable) = self.elements[focused].editable.as_deref_mut() else {
            return false;
        };
        let Some(command) = edit_command(event, editable.line_mode) else {
            return false;
        };
        let Some(change) = editable.perform(command, self.clipboard.as_deref_mut()) else {
            return false;
        };
        self.text_edited_by_user(focused, change, handler);
        true
    }

    /// Lets the pointer select in an edited text: a press of the primary button on a text
    /// that has focus puts the caret there, and the selection follows the pointer from then
    /// until the button is released.
    pub(super) fn select_text_with_pointer(&mut self, input: &PointerInput, hit: Option<ElementId>) {
        match input.kind {
            PointerInputKind::Pressed {
                button: PointerButton::Primary,
                click_count,
            } => {
                let Some(pressed) = hit.filter(|hit| self.focused_text_input() == Some(*hit)) else {
                    return;
                };
                let local = self.point_in_border_box(pressed, input.position);
                let extends = input.modifiers.contains(Modifiers::SHIFT);
                let change = self.editable_mut(pressed).press(local, click_count, extends);
                self.input.text_drag = Some(pressed);
                self.text_edited(pressed, change);
            }
            PointerInputKind::Moved => {
                let Some(dragged) = self.input.text_drag.filter(|dragged| self.elements.contains(*dragged)) else {
                    return;
                };
                let local = self.point_in_border_box(dragged, input.position);
                let change = self.editable_mut(dragged).drag_to(local);
                self.text_edited(dragged, change);
            }
            PointerInputKind::Released {
                button: PointerButton::Primary,
                ..
            } => {
                if let Some(dragged) = self.input.text_drag.take()
                    && self.elements.contains(dragged)
                {
                    self.editable_mut(dragged).end_drag();
                }
            }
            PointerInputKind::Pressed { .. } | PointerInputKind::Released { .. } | PointerInputKind::Exited => {}
        }
    }

    /// Does what an assistive technology asks of an edited text. Returns whether the request
    /// was one an edited text answers.
    pub(super) fn edit_text_by_request(
        &mut self,
        target: ElementId,
        request: &ActionRequest,
        handler: &mut dyn InputHandler,
    ) -> bool {
        let element = &self.elements[target];
        let Some(editable) = element.editable.as_deref() else {
            return false;
        };
        let change = match (request.action, &request.data) {
            (Action::SetTextSelection, Some(ActionData::SetTextSelection(asked))) => {
                let runs = TextRuns {
                    element: target,
                    editable,
                    border_box: element.border_box,
                    on_surface: &Transform::IDENTITY,
                };
                let Some(selection) = runs.selection_of(asked) else {
                    return true;
                };
                self.editable_mut(target).select(selection)
            }
            (Action::ReplaceSelectedText, Some(ActionData::Value(text))) => {
                self.editable_mut(target).insert(text, None)
            }
            (Action::SetValue, Some(ActionData::Value(text))) => {
                let whole = 0..editable.editor.text().len();
                self.editable_mut(target).insert(text, Some(whole))
            }
            _ => return false,
        };
        self.text_edited_by_user(target, change, handler);
        true
    }

    /// Follows the focus of edited texts into the frame at the time of the timeline: a text
    /// shows its selection and its caret only while its element has focus, and the caret
    /// blinks, so the tree asks for the frame in which the caret turns.
    pub(super) fn advance_caret(&mut self) {
        let focused = self.focused_text_input();
        if self.text_focus != focused {
            for shown in [self.text_focus, focused].into_iter().flatten() {
                if self.elements.contains(shown) {
                    self.elements[shown].dirty.insert(Dirty::PAINT);
                    self.has_unpainted_changes = true;
                }
            }
            if let Some(focused) = focused {
                let editable = self.editable_mut(focused);
                editable.blink.restart();
                editable.shows_caret = true;
            }
            self.text_focus = focused;
        }
        let Some(focused) = focused else {
            return;
        };
        let now = self.timeline.now;
        let editable = self.editable_mut(focused);
        let (shows_caret, until_turn) = editable.blink.phase_at(now);
        let turned = editable.shows_caret != shows_caret;
        editable.shows_caret = shows_caret;
        let blinks = editable.editor.selection().is_caret();
        if turned {
            self.elements[focused].dirty.insert(Dirty::PAINT);
            self.has_unpainted_changes = true;
        }
        if blinks {
            self.frame_requests.request_after(until_turn);
        }
    }

    /// Marks what a change of an edited text touches: its lines when its characters changed,
    /// its node in the accessibility tree and what it paints.
    pub(super) fn text_edited(&mut self, element: ElementId, change: EditChange) {
        if change == EditChange::Nothing {
            return;
        }
        let held = &mut self.elements[element];
        held.dirty.insert(Dirty::PAINT);
        if let Some(semantics) = &mut held.semantics {
            semantics.text_changed();
        }
        let is_attached = held.is_attached;
        // Only a text of several lines is as high as its lines; one line never changes its box.
        let resizes_its_box = held.role == Role::TextArea;
        if change == EditChange::Text {
            self.elements.invalidate_text(element);
            if is_attached && resizes_its_box {
                self.elements.invalidate_layout(element);
            }
        }
        if is_attached {
            self.has_unpainted_changes = true;
            self.request_frame();
        }
    }

    /// As [`text_edited`](Self::text_edited), for a change the user made: the element hears
    /// of a text that changed.
    fn text_edited_by_user(&mut self, element: ElementId, change: EditChange, handler: &mut dyn InputHandler) {
        self.text_edited(element, change);
        if change == EditChange::Text {
            handler.text_changed(element, self.text(element));
        }
    }

    fn editable(&self, element: ElementId) -> &EditableText {
        let held = &self.elements[element];
        held.editable
            .as_deref()
            .unwrap_or_else(|| panic!("{:?} edits no text", held.role))
    }

    fn editable_mut(&mut self, element: ElementId) -> &mut EditableText {
        let held = &mut self.elements[element];
        let role = held.role;
        held.editable
            .as_deref_mut()
            .unwrap_or_else(|| panic!("{role:?} edits no text"))
    }

    /// A point of the surface as the element has it, from the top left corner of its border
    /// box, the transforms of the element and of its ancestors taken back.
    fn point_in_border_box(&self, element: ElementId, point: Point) -> Point {
        let in_layout = self
            .surface_transform(element)
            .inverse()
            .map_or(point, |to_layout| to_layout.apply(point));
        let origin = self.elements[element].border_box.origin();
        Point {
            x: in_layout.x - origin.x,
            y: in_layout.y - origin.y,
        }
    }
}
