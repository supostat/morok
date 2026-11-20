/// A column of a pane of the file manager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FileColumn {
    Name,
    Size,
    Modified,
}
