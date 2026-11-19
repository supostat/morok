/// What a toast tells of, shown by the color of its dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ToastTone {
    /// Something was done.
    Success,
    /// Something was done and needs a look.
    Warning,
    /// Something failed.
    Danger,
}
