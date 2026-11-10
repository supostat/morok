use std::ops::Range;

use crate::editing::composition::Composition;
use crate::editing::edit_history::EditHistory;
use crate::editing::edit_kind::EditKind;
use crate::editing::edit_record::EditRecord;
use crate::editing::text_boundaries::floor_boundary;
use crate::editing::text_selection::TextSelection;

/// A plain text as it is edited: its characters, its selection, the text an input method is
/// composing in it, and its history. Places are bytes of the text and lie between grapheme
/// clusters.
#[derive(Debug, Default)]
pub(crate) struct TextEditor {
    text: String,
    selection: TextSelection,
    composition: Option<Composition>,
    history: EditHistory,
}

impl TextEditor {
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn selection(&self) -> TextSelection {
        self.selection
    }

    /// The bytes an input method is composing.
    pub(crate) fn marked(&self) -> Option<Range<usize>> {
        self.composition.as_ref().map(|composition| composition.range.clone())
    }

    /// Replaces the whole text from outside the editing: the caret goes to its end and the
    /// history starts over, as a text that was never edited.
    pub(crate) fn set_text(&mut self, text: &str) {
        self.text.clear();
        self.text.push_str(text);
        self.selection = TextSelection::caret(text.len());
        self.composition = None;
        self.history.clear();
    }

    /// Moves the selection. Both ends fall back on the edge of the grapheme cluster they are
    /// in; a text that is being composed is settled as it stands. Returns whether the
    /// selection is another one now.
    pub(crate) fn select(&mut self, selection: TextSelection) -> bool {
        self.unmark();
        self.history.end_run();
        let selection = TextSelection {
            anchor: floor_boundary(&self.text, selection.anchor),
            head: floor_boundary(&self.text, selection.head),
        };
        let changes = self.selection != selection;
        self.selection = selection;
        changes
    }

    /// Puts `replacement` in place of the bytes of `range` and the caret after it. Returns
    /// whether anything changed.
    pub(crate) fn edit(&mut self, range: Range<usize>, replacement: &str, kind: EditKind) -> bool {
        self.unmark();
        if range.is_empty() && replacement.is_empty() {
            return false;
        }
        let selection_after = TextSelection::caret(range.start + replacement.len());
        let record = EditRecord {
            offset: range.start,
            removed: self.text[range.clone()].to_owned(),
            inserted: replacement.to_owned(),
            selection_before: self.selection,
            selection_after,
        };
        self.text.replace_range(range, replacement);
        self.selection = selection_after;
        self.history.record(record, kind);
        true
    }

    /// Inserts what was typed, dictated or settled on by an input method: in place of the
    /// text being composed when there is one, else of `replacement`, else of the selection.
    pub(crate) fn insert(&mut self, text: &str, replacement: Option<Range<usize>>) -> bool {
        let Some(composition) = self.composition.take() else {
            let range = replacement.map_or(self.selection.range(), |range| self.floor_range(range));
            return self.edit(range, text, EditKind::Typing);
        };
        self.text.replace_range(composition.range.clone(), text);
        self.selection = TextSelection::caret(composition.range.start + text.len());
        self.record_composed(composition, text.to_owned());
        true
    }

    /// Shows what an input method composes: `text` stands marked in place of the text marked
    /// before, else of `replacement`, else of the selection, with `selected` of its bytes
    /// selected. An empty text gives the composition up.
    pub(crate) fn set_marked_text(&mut self, text: &str, selected: Range<usize>, replacement: Option<Range<usize>>) {
        let composition = self.composition.take().unwrap_or_else(|| {
            let range = replacement.map_or(self.selection.range(), |range| self.floor_range(range));
            Composition {
                replaced: self.text[range.clone()].to_owned(),
                selection_before: self.selection,
                range,
            }
        });
        self.history.end_run();
        self.text.replace_range(composition.range.clone(), text);
        let start = composition.range.start;
        self.selection = TextSelection {
            anchor: floor_boundary(&self.text, start + selected.start.min(text.len())),
            head: floor_boundary(&self.text, start + selected.end.min(text.len())),
        };
        if text.is_empty() {
            self.record_composed(composition, String::new());
        } else {
            self.composition = Some(Composition {
                range: start..start + text.len(),
                ..composition
            });
        }
    }

    /// Settles the text being composed as it stands. Returns whether a text was being composed.
    pub(crate) fn unmark(&mut self) -> bool {
        let Some(composition) = self.composition.take() else {
            return false;
        };
        let composed = self.text[composition.range.clone()].to_owned();
        self.record_composed(composition, composed);
        true
    }

    /// Takes the last change back. Returns whether there was one.
    pub(crate) fn undo(&mut self) -> bool {
        self.unmark();
        let Some(record) = self.history.undo() else {
            return false;
        };
        self.text
            .replace_range(record.offset..record.offset + record.inserted.len(), &record.removed);
        self.selection = record.selection_before;
        true
    }

    /// Makes the change that was taken back last again. Returns whether there was one.
    pub(crate) fn redo(&mut self) -> bool {
        self.unmark();
        let Some(record) = self.history.redo() else {
            return false;
        };
        self.text
            .replace_range(record.offset..record.offset + record.removed.len(), &record.inserted);
        self.selection = record.selection_after;
        true
    }

    /// A composition is one step of the history however many times its text changed: what
    /// it replaced gives way to what it ended as.
    fn record_composed(&mut self, composition: Composition, composed: String) {
        if composition.replaced.is_empty() && composed.is_empty() {
            return;
        }
        let record = EditRecord {
            offset: composition.range.start,
            removed: composition.replaced,
            inserted: composed,
            selection_before: composition.selection_before,
            selection_after: self.selection,
        };
        self.history.record(record, EditKind::Single);
    }

    fn floor_range(&self, range: Range<usize>) -> Range<usize> {
        let end = floor_boundary(&self.text, range.end);
        floor_boundary(&self.text, range.start).min(end)..end
    }
}

#[cfg(test)]
mod tests {
    use super::TextEditor;
    use crate::editing::edit_kind::EditKind;
    use crate::editing::text_selection::TextSelection;

    fn editor_holding(text: &str) -> TextEditor {
        let mut editor = TextEditor::default();
        editor.set_text(text);
        editor
    }

    #[test]
    fn a_text_set_from_outside_has_its_caret_at_the_end_and_nothing_to_undo() {
        let mut editor = editor_holding("report");

        assert_eq!(editor.selection(), TextSelection::caret(6));
        assert!(!editor.undo());
        assert_eq!(editor.text(), "report");
    }

    #[test]
    fn a_composition_is_marked_until_it_is_settled_and_is_undone_as_one_step() {
        let mut editor = editor_holding("ab");
        editor.select(TextSelection::caret(1));

        editor.set_marked_text("to", 2..2, None);
        editor.set_marked_text("tou", 3..3, None);
        assert_eq!((editor.text(), editor.marked()), ("atoub", Some(1..4)));
        assert_eq!(editor.selection(), TextSelection::caret(4));

        editor.insert("\u{6771}", None);
        assert_eq!((editor.text(), editor.marked()), ("a\u{6771}b", None));
        assert_eq!(editor.selection(), TextSelection::caret(4));

        assert!(editor.undo());
        assert_eq!((editor.text(), editor.selection()), ("ab", TextSelection::caret(1)));
        assert!(!editor.undo());
    }

    #[test]
    fn a_composition_that_is_given_up_leaves_what_it_replaced_removed() {
        let mut editor = editor_holding("abc");
        editor.select(TextSelection { anchor: 1, head: 2 });

        editor.set_marked_text("x", 1..1, None);
        editor.set_marked_text("", 0..0, None);

        assert_eq!((editor.text(), editor.marked()), ("ac", None));
        assert!(editor.undo());
        assert_eq!(editor.text(), "abc");
        assert_eq!(editor.selection(), TextSelection { anchor: 1, head: 2 });
    }

    #[test]
    fn moving_the_selection_settles_a_composition_as_it_stands() {
        let mut editor = editor_holding("");

        editor.set_marked_text("ni", 2..2, None);
        editor.select(TextSelection::caret(0));

        assert_eq!((editor.text(), editor.marked()), ("ni", None));
        assert!(editor.undo());
        assert_eq!(editor.text(), "");
    }

    #[test]
    fn an_end_of_a_selection_inside_a_grapheme_cluster_falls_back_on_its_start() {
        let mut editor = editor_holding("e\u{301}x");

        editor.select(TextSelection { anchor: 1, head: 9 });

        assert_eq!(editor.selection(), TextSelection { anchor: 0, head: 4 });
    }

    #[test]
    fn an_edit_that_changes_nothing_is_no_step_of_the_history() {
        let mut editor = editor_holding("a");

        assert!(!editor.edit(1..1, "", EditKind::Single));
        assert!(!editor.undo());
    }
}
