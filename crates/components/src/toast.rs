use crate::toast_tone::ToastTone;

/// A short note about what just happened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toast {
    /// Tells the toast from the others of its stack.
    pub id: u64,
    pub title: String,
    /// A second line in small type.
    pub detail: Option<String>,
    pub tone: ToastTone,
}
