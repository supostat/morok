use crate::editing::edit_kind::EditKind;
use crate::editing::edit_record::EditRecord;

/// The changes made to a text, for undo and redo. Typing is undone a run at a time, as the
/// text views of the system undo it: a run ends when the caret is moved, when the kind of the
/// change turns, and at every change that stands by itself.
#[derive(Debug, Default)]
pub(crate) struct EditHistory {
    done: Vec<EditRecord>,
    undone: Vec<EditRecord>,
    /// The kind of the run the last step may still grow by.
    open_run: Option<EditKind>,
}

impl EditHistory {
    /// Notes a change that was made. A change after an undo leaves nothing to redo.
    pub(crate) fn record(&mut self, record: EditRecord, kind: EditKind) {
        self.undone.clear();
        let grows_the_run =
            self.open_run == Some(kind) && self.done.last_mut().is_some_and(|last| last.absorb(&record, kind));
        if !grows_the_run {
            self.done.push(record);
        }
        self.open_run = (kind != EditKind::Single).then_some(kind);
    }

    /// Ends the run: the next change is a step of its own.
    pub(crate) fn end_run(&mut self) {
        self.open_run = None;
    }

    /// The last change that was made, taken back by the caller.
    pub(crate) fn undo(&mut self) -> Option<&EditRecord> {
        self.open_run = None;
        let record = self.done.pop()?;
        self.undone.push(record);
        self.undone.last()
    }

    /// The last change that was taken back, made again by the caller.
    pub(crate) fn redo(&mut self) -> Option<&EditRecord> {
        self.open_run = None;
        let record = self.undone.pop()?;
        self.done.push(record);
        self.done.last()
    }

    pub(crate) fn clear(&mut self) {
        self.done.clear();
        self.undone.clear();
        self.open_run = None;
    }
}
