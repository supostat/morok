//! Plain-text editing: the text of an editable element with its selection and its history, the
//! commands of the keyboard, and the lines the text is laid out, scrolled and painted in.

pub(crate) mod caret_affinity;
pub(crate) mod caret_blink;
pub(crate) mod clipboard;
pub(crate) mod composition;
pub(crate) mod edit_command;
pub(crate) mod edit_history;
pub(crate) mod edit_kind;
pub(crate) mod edit_record;
pub(crate) mod editable_layout;
pub(crate) mod editable_layout_pass;
pub(crate) mod editable_painter;
pub(crate) mod editable_text;
pub(crate) mod key_bindings;
pub(crate) mod line_mode;
pub(crate) mod movement;
pub(crate) mod selection_drag;
pub(crate) mod text_boundaries;
pub(crate) mod text_command;
pub(crate) mod text_editor;
pub(crate) mod text_runs;
pub(crate) mod text_selection;
