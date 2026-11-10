use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use crate::editing::caret_affinity::CaretAffinity;
use crate::editing::caret_blink::CaretBlink;
use crate::editing::clipboard::Clipboard;
use crate::editing::edit_command::EditCommand;
use crate::editing::edit_kind::EditKind;
use crate::editing::editable_layout::{CARET_WIDTH, EditableLayout};
use crate::editing::line_mode::LineMode;
use crate::editing::movement::Movement;
use crate::editing::selection_drag::{DragUnit, SelectionDrag};
use crate::editing::text_boundaries::{
    graphemes_after, next_grapheme, paragraph_end, paragraph_start, previous_grapheme, word_around, word_end_after,
    word_start_before,
};
use crate::editing::text_editor::TextEditor;
use crate::editing::text_selection::TextSelection;
use crate::geometry::point::Point;
use crate::geometry::rect::Rect;

/// What an edit changed of an edited text; the later a kind, the more it changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum EditChange {
    Nothing,
    /// What the text shows changed — its selection or how far it is scrolled — and its
    /// characters did not.
    View,
    Text,
}

/// The edited text of an element: the text with its selection and history, its placeholder,
/// the lines it was last laid out in and how far they are scrolled inside the content box.
///
/// Commands that go by lines read the lines of the last layout. Until a text that changed is
/// laid out again its lines are not known, and such a command takes every paragraph for one
/// line and every character for as wide as the next.
#[derive(Debug)]
pub(crate) struct EditableText {
    pub(crate) line_mode: LineMode,
    pub(crate) editor: TextEditor,
    pub(crate) placeholder: String,
    pub(crate) layout: EditableLayout,
    /// How far the lines are moved left and up inside the content box.
    pub(crate) scroll: Point,
    /// The column that successive moves from line to line keep: a distance from the left of
    /// the content box.
    goal_x: Option<f32>,
    affinity: CaretAffinity,
    /// Counts the changes of the text.
    pub(crate) text_revision: u64,
    /// Whether the next layout of the lines scrolls the caret into view.
    pub(crate) reveals_caret: bool,
    pub(crate) blink: CaretBlink,
    /// Whether the caret is in the shown half of its blinking.
    pub(crate) shows_caret: bool,
    drag: Option<SelectionDrag>,
}

impl EditableText {
    pub(crate) fn new(line_mode: LineMode) -> Self {
        Self {
            line_mode,
            editor: TextEditor::default(),
            placeholder: String::new(),
            layout: EditableLayout::default(),
            scroll: Point::default(),
            goal_x: None,
            affinity: CaretAffinity::Downstream,
            text_revision: 0,
            reveals_caret: false,
            blink: CaretBlink::default(),
            shows_caret: true,
            drag: None,
        }
    }

    pub(crate) fn affinity(&self) -> CaretAffinity {
        self.affinity
    }

    /// Whether the text scrolls sideways: one line does, and lines that wrap at the width of
    /// the box never reach past it.
    pub(crate) fn scrolls_sideways(&self) -> bool {
        self.line_mode == LineMode::Single
    }

    /// Whether the lines of the last layout are those of the text as it is now.
    pub(crate) fn has_current_lines(&self) -> bool {
        self.layout.text_revision == Some(self.text_revision)
    }

    /// Replaces the whole text from outside the editing. Returns whether it changed.
    pub(crate) fn set_text(&mut self, text: &str) -> EditChange {
        let text = self.line_mode.sanitized(text);
        if self.editor.text() == text {
            return EditChange::Nothing;
        }
        self.goal_x = None;
        self.editor.set_text(&text);
        self.note(EditChange::Text, CaretAffinity::Downstream)
    }

    /// Moves the selection from outside the editing.
    pub(crate) fn select(&mut self, selection: TextSelection) -> EditChange {
        self.goal_x = None;
        let change = self.selected(selection);
        self.note(change, CaretAffinity::Downstream)
    }

    /// Inserts what was typed, dictated or settled on by an input method.
    pub(crate) fn insert(&mut self, text: &str, replacement: Option<Range<usize>>) -> EditChange {
        self.goal_x = None;
        let text = self.line_mode.sanitized(text);
        let change = text_change(self.editor.insert(&text, replacement));
        self.note(change, CaretAffinity::Downstream)
    }

    /// Shows the text an input method composes.
    pub(crate) fn set_marked_text(
        &mut self,
        text: &str,
        selected: Range<usize>,
        replacement: Option<Range<usize>>,
    ) -> EditChange {
        self.goal_x = None;
        let text = self.line_mode.sanitized(text);
        self.editor.set_marked_text(&text, selected, replacement);
        self.note(EditChange::Text, CaretAffinity::Downstream)
    }

    /// Settles the text an input method composes as it stands.
    pub(crate) fn unmark(&mut self) -> EditChange {
        let change = if self.editor.unmark() {
            EditChange::View
        } else {
            EditChange::Nothing
        };
        self.note(change, self.affinity)
    }

    /// Does what the keyboard asked. Returns none for a command that could not be heard: one
    /// of the clipboard when the tree has none.
    pub(crate) fn perform(
        &mut self,
        command: EditCommand,
        clipboard: Option<&mut (dyn Clipboard + 'static)>,
    ) -> Option<EditChange> {
        if !matches!(command, EditCommand::Move(movement) | EditCommand::Extend(movement) if movement.is_vertical()) {
            self.goal_x = None;
        }
        let selection = self.editor.selection();
        let mut affinity = CaretAffinity::Downstream;
        let change = match command {
            EditCommand::Move(movement) => {
                let range = selection.range();
                let collapses =
                    !selection.is_caret() && matches!(movement, Movement::GraphemeBackward | Movement::GraphemeForward);
                let target = if collapses {
                    if movement.is_backward() { range.start } else { range.end }
                } else {
                    let from = if movement.is_backward() { range.start } else { range.end };
                    let (target, target_affinity) = self.target(movement, from);
                    affinity = target_affinity;
                    target
                };
                self.selected(TextSelection::caret(target))
            }
            EditCommand::Extend(movement) => {
                let (head, head_affinity) = self.target(movement, selection.head);
                affinity = head_affinity;
                self.selected(TextSelection {
                    anchor: selection.anchor,
                    head,
                })
            }
            EditCommand::Delete(movement) => {
                let range = if selection.is_caret() {
                    let (target, _) = self.target(movement, selection.head);
                    target.min(selection.head)..target.max(selection.head)
                } else {
                    selection.range()
                };
                let kind = match movement {
                    Movement::GraphemeBackward if selection.is_caret() => EditKind::DeletingBackward,
                    Movement::GraphemeForward if selection.is_caret() => EditKind::DeletingForward,
                    _ => EditKind::Single,
                };
                text_change(self.editor.edit(range, "", kind))
            }
            EditCommand::InsertLineBreak => text_change(self.editor.edit(selection.range(), "\n", EditKind::Typing)),
            EditCommand::OpenLine => {
                let range = selection.range();
                let caret = TextSelection::caret(range.start);
                let changed = self.editor.edit(range, "\n", EditKind::Single);
                self.editor.select(caret);
                text_change(changed)
            }
            EditCommand::Transpose => self.transpose(),
            EditCommand::SelectAll => {
                let end = self.editor.text().len();
                self.selected(TextSelection { anchor: 0, head: end })
            }
            EditCommand::Copy => {
                self.copy_selection(clipboard?);
                EditChange::Nothing
            }
            EditCommand::Cut => {
                self.copy_selection(clipboard?);
                text_change(self.editor.edit(selection.range(), "", EditKind::Single))
            }
            EditCommand::Paste => match clipboard?.text() {
                Some(pasted) => {
                    let pasted = self.line_mode.sanitized(&pasted);
                    text_change(self.editor.edit(selection.range(), &pasted, EditKind::Single))
                }
                None => EditChange::Nothing,
            },
            EditCommand::Undo => text_change(self.editor.undo()),
            EditCommand::Redo => text_change(self.editor.redo()),
            EditCommand::ScrollToStart => return Some(self.scroll_to(Point::default())),
            EditCommand::ScrollToEnd => {
                return Some(self.scroll_to(Point {
                    x: f32::MAX,
                    y: f32::MAX,
                }));
            }
            EditCommand::ScrollPageUp => return Some(self.scroll_by_page(-1.0)),
            EditCommand::ScrollPageDown => return Some(self.scroll_by_page(1.0)),
        };
        Some(self.note(change, affinity))
    }

    /// The pointer went down at a point of the border box: one press puts the caret there,
    /// or with `extends` the head of the selection; two presses select the word there and
    /// three the paragraph. The selection is drawn from there until the pointer is released.
    pub(crate) fn press(&mut self, local: Point, click_count: u32, extends: bool) -> EditChange {
        let (place, affinity) = self.place_at(local);
        let unit = DragUnit::of_click_count(click_count);
        let anchor = self.editor.selection().anchor;
        let origin = match unit {
            DragUnit::Character if extends => anchor..anchor,
            DragUnit::Character => place..place,
            DragUnit::Word => word_around(self.editor.text(), place),
            DragUnit::Paragraph => paragraph_start(self.editor.text(), place)..paragraph_end(self.editor.text(), place),
        };
        let selection = match unit {
            DragUnit::Character => TextSelection {
                anchor: origin.start,
                head: place,
            },
            DragUnit::Word | DragUnit::Paragraph => TextSelection {
                anchor: origin.start,
                head: origin.end,
            },
        };
        self.drag = Some(SelectionDrag { origin, unit });
        self.goal_x = None;
        let change = self.selected(selection);
        self.note(change, affinity)
    }

    /// The pointer moved to a point of the border box with the button held: the selection
    /// reaches from what the press selected to there.
    pub(crate) fn drag_to(&mut self, local: Point) -> EditChange {
        let Some(SelectionDrag { origin, unit }) = self.drag.clone() else {
            return EditChange::Nothing;
        };
        let (place, affinity) = self.place_at(local);
        let text = self.editor.text();
        let reached = match unit {
            DragUnit::Character => place..place,
            DragUnit::Word => word_around(text, place),
            DragUnit::Paragraph => paragraph_start(text, place)..paragraph_end(text, place),
        };
        let selection = if reached.start < origin.start {
            TextSelection {
                anchor: origin.end,
                head: reached.start,
            }
        } else {
            TextSelection {
                anchor: origin.start,
                head: reached.end.max(origin.end),
            }
        };
        let change = self.selected(selection);
        self.note(change, affinity)
    }

    /// The pointer was released: the selection is drawn.
    pub(crate) fn end_drag(&mut self) {
        self.drag = None;
    }

    /// The place of the text nearest to a point of the border box.
    pub(crate) fn offset_at(&self, local: Point) -> usize {
        self.place_at(local).0
    }

    /// The box of the first line of a range of the text, relative to the border box. Without
    /// current lines it is the content box.
    pub(crate) fn range_rect(&self, range: Range<usize>) -> Rect {
        if !self.has_current_lines() {
            return self.layout.content_box;
        }
        let layout = &self.layout;
        let line = layout.line_of(range.start, CaretAffinity::Downstream);
        let end = range.end.min(layout.caret_range(line).end);
        let (left, right) = (layout.x_of(line, range.start), layout.x_of(line, end));
        Rect::new(
            layout.content_box.x + layout.line_left(line) + left - self.scroll.x,
            layout.content_box.y + layout.line_top(line) - self.scroll.y,
            right - left,
            layout.line_height,
        )
    }

    /// Scrolls the lines the least that brings the caret into the content box. Called once
    /// the lines are those of the text as it is.
    pub(crate) fn reveal_caret(&mut self) {
        self.reveals_caret = false;
        let layout = &self.layout;
        let head = self.editor.selection().head;
        let line = layout.line_of(head, self.affinity);
        let caret_left = layout.line_left(line) + layout.x_of(line, head);
        let line_top = layout.line_top(line);
        let view = layout.content_box.size();
        if self.scrolls_sideways() {
            self.scroll.x = self.scroll.x.max(caret_left + CARET_WIDTH - view.width).min(caret_left);
        }
        self.scroll.y = self
            .scroll
            .y
            .max(line_top + layout.line_height - view.height)
            .min(line_top);
        self.keep_scroll_in_range();
    }

    /// Keeps the lines from being scrolled past their ends: a text that got shorter moves
    /// back into the box.
    pub(crate) fn keep_scroll_in_range(&mut self) {
        let layout = &self.layout;
        let view = layout.content_box.size();
        let furthest_left = if self.scrolls_sideways() {
            (layout.text_width() + CARET_WIDTH - view.width).max(0.0)
        } else {
            0.0
        };
        let furthest_up = (layout.text_height() - view.height).max(0.0);
        self.scroll.x = self.scroll.x.clamp(0.0, furthest_left);
        self.scroll.y = self.scroll.y.clamp(0.0, furthest_up);
    }

    /// Notes what a change did to the text: a caret that moved is shown at once, and is
    /// scrolled into view — now when the lines are current, else when they are laid out.
    fn note(&mut self, change: EditChange, affinity: CaretAffinity) -> EditChange {
        if change == EditChange::Nothing {
            return change;
        }
        self.affinity = affinity;
        self.blink.restart();
        self.shows_caret = true;
        if change == EditChange::Text {
            self.text_revision += 1;
        }
        if self.has_current_lines() {
            self.reveal_caret();
        } else {
            self.reveals_caret = true;
        }
        change
    }

    fn selected(&mut self, selection: TextSelection) -> EditChange {
        if self.editor.select(selection) {
            EditChange::View
        } else {
            EditChange::Nothing
        }
    }

    fn copy_selection(&self, clipboard: &mut dyn Clipboard) {
        let selected = &self.editor.text()[self.editor.selection().range()];
        if !selected.is_empty() {
            clipboard.set_text(selected);
        }
    }

    /// Swaps the characters on either side of the caret and steps over them; at the end of a
    /// paragraph the two characters before the caret are swapped, as the text views of the
    /// system swap them.
    fn transpose(&mut self) -> EditChange {
        let selection = self.editor.selection();
        let text = self.editor.text();
        if !selection.is_caret() {
            return EditChange::Nothing;
        }
        let caret = selection.head;
        let at_paragraph_end = caret == paragraph_end(text, caret);
        let middle = if at_paragraph_end {
            previous_grapheme(text, caret)
        } else {
            caret
        };
        let (start, end) = (previous_grapheme(text, middle), next_grapheme(text, middle));
        let stays_in_paragraph = start >= paragraph_start(text, caret) && end <= paragraph_end(text, caret);
        if start == middle || middle == end || !stays_in_paragraph {
            return EditChange::Nothing;
        }
        let swapped = format!("{}{}", &text[middle..end], &text[start..middle]);
        text_change(self.editor.edit(start..end, &swapped, EditKind::Single))
    }

    fn scroll_to(&mut self, scroll: Point) -> EditChange {
        if !self.has_current_lines() {
            return EditChange::Nothing;
        }
        let before = self.scroll;
        self.scroll = scroll;
        self.keep_scroll_in_range();
        if self.scroll == before {
            EditChange::Nothing
        } else {
            EditChange::View
        }
    }

    fn scroll_by_page(&mut self, pages: f32) -> EditChange {
        let page = self.layout.lines_in_view() as f32 * self.layout.line_height;
        self.scroll_to(Point {
            x: self.scroll.x,
            y: self.scroll.y + pages * page,
        })
    }

    /// Where a movement takes a caret from `from`, and which of two lines the caret belongs
    /// to there when a line wraps at that place.
    fn target(&mut self, movement: Movement, from: usize) -> (usize, CaretAffinity) {
        let text = self.editor.text();
        let downstream = |offset: usize| (offset, CaretAffinity::Downstream);
        match movement {
            Movement::GraphemeBackward => downstream(previous_grapheme(text, from)),
            Movement::GraphemeForward => downstream(next_grapheme(text, from)),
            Movement::WordBackward => downstream(word_start_before(text, from)),
            Movement::WordForward => downstream(word_end_after(text, from)),
            Movement::ParagraphStart => downstream(paragraph_start(text, from)),
            Movement::ParagraphEnd => downstream(paragraph_end(text, from)),
            Movement::ParagraphBackward => downstream(paragraph_start(text, previous_grapheme(text, from))),
            Movement::ParagraphForward => downstream(paragraph_end(text, next_grapheme(text, from))),
            Movement::DocumentStart => downstream(0),
            Movement::DocumentEnd => downstream(text.len()),
            Movement::LineStart => downstream(self.line_caret_range(from).start),
            Movement::LineEnd => self.line_end(from),
            Movement::LineUp => self.lines_away(from, -1),
            Movement::LineDown => self.lines_away(from, 1),
            Movement::PageUp => self.lines_away(from, -self.page_lines()),
            Movement::PageDown => self.lines_away(from, self.page_lines()),
        }
    }

    fn page_lines(&self) -> isize {
        if self.has_current_lines() {
            isize::try_from(self.layout.lines_in_view()).expect("a box shows fewer lines than fit in an offset")
        } else {
            1
        }
    }

    /// The places a caret may stand at in the line `from` is in.
    fn line_caret_range(&self, from: usize) -> Range<usize> {
        if self.has_current_lines() {
            self.layout.caret_range(self.layout.line_of(from, self.affinity))
        } else {
            let text = self.editor.text();
            paragraph_start(text, from)..paragraph_end(text, from)
        }
    }

    /// The end of the line `from` is in. Where the line wraps, its end is also the start of
    /// the next line, and the caret stays with the line it came from.
    fn line_end(&self, from: usize) -> (usize, CaretAffinity) {
        let end = self.line_caret_range(from).end;
        if !self.has_current_lines() {
            return (end, CaretAffinity::Downstream);
        }
        let line = self.layout.line_of(from, self.affinity);
        if self.layout.wraps_into_next(line) {
            (end, CaretAffinity::Upstream)
        } else {
            (end, CaretAffinity::Downstream)
        }
    }

    /// The place `lines` lines below `from`, above it when negative, in the column the caret
    /// keeps. Past the first line it is the start of the text and past the last its end.
    fn lines_away(&mut self, from: usize, lines: isize) -> (usize, CaretAffinity) {
        if !self.has_current_lines() {
            return (self.paragraphs_away(from, lines), CaretAffinity::Downstream);
        }
        let layout = &self.layout;
        let line = layout.line_of(from, self.affinity);
        let goal_x = *self
            .goal_x
            .get_or_insert_with(|| layout.line_left(line) + layout.x_of(line, from));
        let Some(target_line) = line.checked_add_signed(lines) else {
            return (0, CaretAffinity::Downstream);
        };
        if target_line >= layout.line_count() {
            return (self.editor.text().len(), CaretAffinity::Downstream);
        }
        self.place_in_line(target_line, goal_x - layout.line_left(target_line))
    }

    /// The place of a line nearest to a point `x` from its start. The end of a line that
    /// wraps belongs to that line.
    fn place_in_line(&self, line: usize, x: f32) -> (usize, CaretAffinity) {
        let layout = &self.layout;
        let offset = layout.offset_at(line, x);
        let is_wrapped_end = layout.wraps_into_next(line) && offset == layout.caret_range(line).end;
        if is_wrapped_end {
            (offset, CaretAffinity::Upstream)
        } else {
            (offset, CaretAffinity::Downstream)
        }
    }

    /// Without lines: the place as many characters into the paragraph `paragraphs` away as
    /// `from` is into its own.
    fn paragraphs_away(&self, from: usize, paragraphs: isize) -> usize {
        let text = self.editor.text();
        let column = text[paragraph_start(text, from)..from].graphemes(true).count();
        let mut place = from;
        for _ in 0..paragraphs.unsigned_abs() {
            let (start, end) = (paragraph_start(text, place), paragraph_end(text, place));
            if paragraphs < 0 {
                if start == 0 {
                    return 0;
                }
                place = start - 1;
            } else {
                if end == text.len() {
                    return text.len();
                }
                place = end + 1;
            }
        }
        graphemes_after(text, paragraph_start(text, place), paragraph_end(text, place), column)
    }

    /// The place of the text nearest to a point of the border box, with the line it lies in.
    /// Without current lines every point is the end of the text.
    fn place_at(&self, local: Point) -> (usize, CaretAffinity) {
        if !self.has_current_lines() {
            return (self.editor.text().len(), CaretAffinity::Downstream);
        }
        let layout = &self.layout;
        let x = local.x - layout.content_box.x + self.scroll.x;
        let y = local.y - layout.content_box.y + self.scroll.y;
        let line = layout.line_at(y);
        self.place_in_line(line, x - layout.line_left(line))
    }
}

fn text_change(changed: bool) -> EditChange {
    if changed { EditChange::Text } else { EditChange::Nothing }
}
