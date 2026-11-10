use crate::editing::edit_kind::EditKind;
use crate::editing::text_selection::TextSelection;

/// One step of the history of a text: what was put in place of what, and the selection on
/// either side of the change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EditRecord {
    /// Where the change begins in the text.
    pub(crate) offset: usize,
    pub(crate) removed: String,
    pub(crate) inserted: String,
    pub(crate) selection_before: TextSelection,
    pub(crate) selection_after: TextSelection,
}

impl EditRecord {
    /// Takes `next` into this step when it goes on where this one ended: typing after what
    /// was typed, a deletion next to what was deleted. Returns whether it did.
    pub(crate) fn absorb(&mut self, next: &Self, kind: EditKind) -> bool {
        match kind {
            EditKind::Typing if next.removed.is_empty() && next.offset == self.offset + self.inserted.len() => {
                self.inserted.push_str(&next.inserted);
            }
            EditKind::DeletingBackward
                if self.inserted.is_empty() && next.offset + next.removed.len() == self.offset =>
            {
                self.removed.insert_str(0, &next.removed);
                self.offset = next.offset;
            }
            EditKind::DeletingForward if self.inserted.is_empty() && next.offset == self.offset => {
                self.removed.push_str(&next.removed);
            }
            EditKind::Typing | EditKind::DeletingBackward | EditKind::DeletingForward | EditKind::Single => {
                return false;
            }
        }
        self.selection_after = next.selection_after;
        true
    }
}
